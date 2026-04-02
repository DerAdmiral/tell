use std::{fs::{DirEntry, Permissions}, ops::ControlFlow, os::unix::fs::PermissionsExt};

use clap::builder::Str;

use crate::tokenizer::Criteria;

// mode permissions bits according to inode (7)
const SUID_BIT: u32 = 0o4000;
const GUID_BIT: u32 = 0o2000;
const STICKY_BIT: u32 = 0o1000;

const UR_BIT: u32 = 0o400;
const UW_BIT: u32 = 0o200;
const UX_BIT: u32 = 0o100;

const GR_BIT: u32 = 0o40;
const GW_BIT: u32 = 0o20;
const GX_BIT: u32 = 0o10;

const OR_BIT: u32 = 0o4;
const OW_BIT: u32 = 0o2;
const OX_BIT: u32 = 0o1;

const AR_BIT: u32 = UR_BIT | GR_BIT | OR_BIT;
const AW_BIT: u32 = UW_BIT | GX_BIT | OW_BIT;
const AX_BIT: u32 = UX_BIT | GX_BIT | OX_BIT;



#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CmpOp {
    Equals,
    Greater,
    Smaller,
    GreaterEquals,
    SmallerEquals,
    NotEquals,
}

pub(crate) fn resolve_multiplier(suffix: &str) -> Result<u64, String> {
    dbg!(match suffix.to_lowercase().as_str() {
        "" => Ok(1),
        "kb" => Ok(u64::pow(10, 3)),
        "kib" => Ok(u64::pow(2,10)),
        "mb" => Ok(u64::pow(10,6)),
        "mib" => Ok(u64::pow(2,20)),
        "gb" => Ok(u64::pow(10,9)),
        "gib" => Ok(u64::pow(2,30)),
        _ => Err(format!("unknown suffix: {suffix} "))
    })
}

pub(crate) fn parse_size(size: &str) -> Result<u64, String> {
    if size.is_empty() {
        return Err("No size defined".to_string());
    }

    let suffix_start = size.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(size.len());

    let (digits, suffix) = size.split_at(suffix_start);   

    let base = digits.parse::<f64>().map_err(|_| format!("Invalid number in: {size}"))?;

    if base < 0f64 || base.is_infinite() || base.is_nan() {
        return Err(format!("Invalid value in: {size}"))
    }

    Ok((base * resolve_multiplier(suffix)? as f64).round() as u64)
}

pub(crate) fn parse_op(constraint: &str) -> Result<(CmpOp, &str), String> {
        Ok(
            match &constraint[0..2] {
                "<=" => (CmpOp::SmallerEquals, &constraint[2..]),
                ">=" => (CmpOp::GreaterEquals, &constraint[2..]),
                "!=" => (CmpOp::NotEquals, &constraint[2..]),
                _ => match &constraint[0..1] {
                    ">" => (CmpOp::Greater, &constraint[1..]),
                    "<" => (CmpOp::Smaller, &constraint[1..]),
                    "=" => (CmpOp::Equals, &constraint[1..]),
                    _ => return Err(format!("Unknown Comparison Operator in size constraint: {constraint} "))
        }
    })
}

pub(crate) fn apply_op(op: CmpOp, size: u64) -> impl Fn(u64) -> bool {
    move |x| match op {
        CmpOp::Equals        => x == size,
        CmpOp::NotEquals     => x != size,
        CmpOp::SmallerEquals => x <= size,
        CmpOp::GreaterEquals => x >= size,
        CmpOp::Smaller       => x <  size,
        CmpOp::Greater       => x >  size,
    }
}

pub(crate) fn parse_misc(property: &str) -> Result<Box<dyn Fn(&DirEntry) -> bool>, String> {
    Ok( Box::new(match property {
        "hidden" => move |entry: &DirEntry| entry.file_name().to_str().map_or(false, |name| name.starts_with('.') && (name != "..") && (name != ".") ), 
        "empty" => move |entry: &DirEntry| entry.metadata().map_or(false, |meta| meta.len() == 0),
        _ => return Err(format!("Unknown property: {property}")),
    }))
}

#[derive(Debug)]
enum PermMatchType {
    AtLeast,
    Exact,
}

#[derive(Debug,PartialEq)]
enum FlagFor {
    User,
    Group,
    Other,
    All,
}

impl FlagFor {
    fn to_bit_flag(&self, c: char) -> Result<u32, String> {
        match (c,self) {
            ('r', FlagFor::User) => Ok(UR_BIT), 
            ('w', FlagFor::User) => Ok(UW_BIT), 
            ('x', FlagFor::User) => Ok(UX_BIT), 
            ('s', FlagFor::User) => Ok(SUID_BIT | UX_BIT), 
            ('S', FlagFor::User) => Ok(SUID_BIT),
            ('r', FlagFor::Group) => Ok(GR_BIT), 
            ('w', FlagFor::Group) => Ok(GW_BIT), 
            ('x', FlagFor::Group) => Ok(GX_BIT), 
            ('s', FlagFor::Group) => Ok(GUID_BIT | GX_BIT), 
            ('S', FlagFor::Group) => Ok(GUID_BIT),
            ('r', FlagFor::Other) => Ok(OR_BIT),
            ('w', FlagFor::Other) => Ok(OW_BIT),
            ('x', FlagFor::Other) => Ok(OX_BIT),
            ('t', FlagFor::Other) => Ok(STICKY_BIT | OX_BIT),
            ('T', FlagFor::Other) => Ok(STICKY_BIT),
            ('r', FlagFor::All) => Ok(AR_BIT),
            ('w', FlagFor::All) => Ok(AW_BIT),
            ('x', FlagFor::All) => Ok(AX_BIT),
            _ => Err(format!("Invalid combination: {:?} and {c}", self))
        }
    }
}


fn create_mask(permission: &str, flagfor: FlagFor) -> Result<u32, String> {
    let mut mode_bits = 0;
    for new_flag in permission.chars().map(|c| flagfor.to_bit_flag(c)) {
        match new_flag {
            Ok(flag) => mode_bits |= flag,
            Err(e) => return Err(e),
        }
    };
    Ok(mode_bits)        
}

pub(crate) fn create_perm_mode_bits(permissions: &str) -> Result<u32, String> {
    // attempt to parse permission bits, assuming its in octal notation. First character indicates if its in octal or not 
    // in case its a digit, the parser assumes the user intended to provide an octal string
    let mut  mode_bits = match u32::from_str_radix(permissions, 8) {
        Ok(mode_bits) => return Ok(mode_bits),
        Err(e) if permissions.starts_with(|c: char| c.is_ascii_digit()) => return Err(format!("Error parsing mode bits: {e}")),
        Err(_) => 0,
    };

    // string was not in octal notation, following structure is expected: a:<perms>,u:<perms>,g:<perms>,o:<perms>
    // the exact order doesnt matter only the prefix and the seperation by comma is important 
    for unparsed_part in permissions.split(',') {
        if let Some((prefix, perms)) = unparsed_part.split_once(':') {
            if perms.is_empty() && !prefix.is_empty(){
                return Err(format!("Missing permissions for {prefix} in: '{permissions}'"))
            }
            if perms.is_empty() {
                return Err(format!("Missing prefix and permissions in '{permissions}'"));
            }
            match prefix {
                "a" => mode_bits |= create_mask(perms, FlagFor::All)?,
                "u" => mode_bits |= create_mask(perms, FlagFor::User)?,
                "g" => mode_bits |= create_mask(perms, FlagFor::Group)?,
                "o" => mode_bits |= create_mask(perms, FlagFor::Other)?,
                c => return Err(format!("unexpected character ({c}) in permission bit: {permissions}"))
            }
            continue;
        }
        return Err(format!("malformed permissions: {unparsed_part} ind {permissions}"))
    };

    Ok(dbg!(mode_bits))

}

pub(crate) fn parse_perm(permissions: &str) -> Result<Box<dyn Fn(&DirEntry) -> bool>, String> {
    if permissions.is_empty() {
        return Err("Empty permission string provided, in case you wanted to express that no permissions are set, use perm:0".to_string())
    }

    let mut permissions = permissions;

    let match_type = match &permissions[0..1] {
        "=" => {permissions = permissions.strip_prefix('=').unwrap(); crate::constraints::PermMatchType::Exact},
        _ => crate::constraints::PermMatchType::AtLeast,
    };

    let mode_bits = create_perm_mode_bits(permissions)?;

    match dbg!(match_type) {
        PermMatchType::Exact => Ok(Box::new(move |entry: &DirEntry| entry.metadata().and_then(|md| Ok(md.permissions().mode() & 0xfff == mode_bits)).unwrap_or(false))), // we only want the 12 lest significant bits, which are the permissions
        PermMatchType::AtLeast => Ok(Box::new(move |entry: &DirEntry| entry.metadata().and_then(|md| Ok((md.permissions().mode() & 0xfff) & mode_bits == mode_bits)).unwrap_or(false))),
    }
}