# Type conversion example in Boba

fun main(): null {
    # Variable declarations
    number = 42
    pi = 3.14159
    
    # Type conversion using function-like syntax
    float_number = float(number)
    output("Integer converted to float:", float_number)
    
    int_pi = int(pi)
    output("Float converted to integer:", int_pi)
    
    text = string(number)
    output("Integer converted to string:", text)
    
    return null
}
