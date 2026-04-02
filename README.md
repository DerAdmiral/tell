# Tell
## State of the project

Currently only the -find option works, the rest is hard coded so -A and -a dont do anything.
As for the search criteria everything works (at least what I've tested and "it works on my machine" ;) - for real these should work) except the time-based ones (atime, mtime, ctime).
Originally I had a similar CLI tool I wanted to release, however I decided to rewrite it and
this is how far I've gotten so far.

## Documentation 

Regarding -a and -A these are the same as for the ls command.
Now to -find option:
  it expects a predicate where every operand and operator is seperated by exactly one whitespace. 
  The predicate has to be in polish notation
  <OPERATOR> <OPERAND1> <OPERAND2>
  E.g. 
    ! size:>123
    & size:>123 ! name:foo.bar
  Available Operators are:
  
  ! negation
  & and
  | or
  ^ xor (either one but not both)
  ~ conditional
  = biconditional

  They work just like the boolean operators.

  One specifies a constraint in the following syntax:
  <criteria>:<constraint string>
  E.g. 'size:>123' (a file larger than 123 bytes)
  
  Available search criteria are (only listing working ones):
  size:
    Filters entrys based on the their size
    The general structure is: <comparison operator><number><suffix>
    Comparison operators:
      >=
      <= 
      !=
      = (exactly)
      >
      <

    The number has to be in base 10
    
    Suffix:
       kB -> kilobyte 
       kiB -> Kibibyte
       MB
       MiB
       GB
       GiB
    
    
  name:
    this criteria expects a regex, returning true for ever entry where it matches the name 
    
  perm:
    filters entries based on their permissions, please refer to inode (7) for additional information
    Two options are available, a leading = indicates that the permissions have to match exactly, otherwise 
    the provided permission are interpreted as minimal requirementes, if additional permissions are set
    the entry is included.
    E.g.: 
      =700 matches only rwx------
      744 matches rwxr--r-- but also rwxr-Srw- 
    Octal argument: 
      Up to four octal digits, leading zeros can be omitted
      Left to right:
      1. digit: sticky bit, set uid bit, set guid bit
      2. digit: user permissions
      3. digit: group permissions
      4. digit: other permissions
    
  type:
    Filters entries based on their type. multiple types can be combined into one string.
    f: file
    d: directory
    l: symbolic link
    c: character device
    b: block device
    s: sockets
    p: named pipes
  E.g.:
    type:f matches any file but excludes everyother type.
    type:fd matches files and directories
  misc:
    simple properties.
    empty: empty files
    hidden: hidden files
    
  ext:
    matches for file extensions, thus excluding everthing not having an extension  
  
