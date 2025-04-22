
let name = open Cargo.toml | get package | get name

mkdir $name

git ls-files
  | lines
  | str trim
  | par-each {|x| if ($x | str contains '/') {
      parse "{name}/{other}" | get name | first
    } else {
      $x
    }
  }
  | uniq 
  | par-each {|x| mv $x $name}

