# Boba Programming Language (In Development)

Boba is a statically typed programming language with a clean and expressive syntax.

## Features

- Static typing with type inference
- First-class functions
- Lists and maps as built-in data structures
- Simple and intuitive syntax
- Built-in I/O operations

## Example

```boba
fun myFunction(mynum: int, mystring: string, myfloat: float): int, string, float {
    foo = 1
    bar = "string"
    myfloat = 12.02102
    mybool = true
    noval = null
    mylist = [1,2,3,4,5]
    mymap = [1:"var",2:"var2"]
    footofloat = foo.float # turn int to float 
    
    #this is a comment
    ###
        this is a multiline comment
    ###
    
    output("hello world") #new line by default
    output("hello world", foo.float + myfloat , "another message")
    outputf("hello {mystring}") #formatted output and putting using mystring as variable
    output&(mynum) #output address of variable
    
    input("get num from user:")
    inputf("get num from {mystring}:")
    
    if foo is int {
        return mybool
    }
    elseif foo is not int {
        output("do nothing")
    }
    else {
        return null
    }
    
    loop i=0,i...100 { # for loop in range to 100, loops count as while too
        outputf({i})
    }
    
    loop i till foo is null {
        continue #continues down to next 
    }
    
    loop i { #while loop example loop i if true
        # do something 
        i = false
    }
}
```

## Getting Started

### Prerequisites

- Rust (latest stable version)

### Building

```bash
cargo build --release
```

### Running a Boba Program

```bash
./target/release/boba run example.bb
```

## Project Structure

- `src/lexer.rs`: Tokenizes the source code
- `src/parser.rs`: Parses tokens into an abstract syntax tree
- `src/ast.rs`: Defines the abstract syntax tree structures
- `src/types.rs`: Implements the type system
- `src/interpreter.rs`: Executes the parsed program
- `src/error.rs`: Error handling utilities

## License

This project is licensed under the MIT License - see the LICENSE file for details.
