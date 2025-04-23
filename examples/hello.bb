# Hello World example in Boba

 

fun main(): null {
    output("Hello, World!")
    
    name = "Boba"
    outputf("Welcome to {name} programming language!")
    
    # Variable declarations
    number = 42
    pi = 3.14159
    is_awesome = true
    
    output("Here are some values:", number, pi, is_awesome)
    
    # List example
    my_list = [1, 2, 3, 4, 5]
    output("A list:", my_list)
    
    # Map example
    my_map = ["name": "Boba", "type": "language"]
    output("A map:", my_map)
    
    # Type conversion
    float_number = float(number)
    output("Converted to float:", float_number)
    
    # Conditional
    if number is int {
        output("number is an integer")
    }
    
    # Loop
    output("Counting to 5:")
    loop i=1, i...6 {
        outputf("{i}")
    }
    return null
}
