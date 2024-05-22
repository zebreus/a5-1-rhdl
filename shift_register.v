
function  [4:0] shift_register_update_4f011f2a5bb660ca(input reg  [3:0] r0, input reg  [3:0] r1, input reg  [0:0] r2);
    // Registers
    reg  [3:0] r11;
    reg  [0:0] r13;
    reg  [3:0] r14;
    reg  [4:0] r16;
    // Literals
    localparam l2 = 4'b0001;
    localparam l3 = 8'b00000011;
    localparam l4 = 8'b00000000;
    // Body
    begin
        // let new_state /* b4 */: b4 /* b4 */ = state << bits<b4>(1, );
        r11 = r1 << l2;
        // let output_bit /* b1 */ = get_bit<b1>(state, 3, );
        r13 = get_bit_4(r1, l3);
        // new_state /*b4*/ = set_bit<b4>(new_state, 0, input, );
        r14 = set_bit_4(r11, l4, r2);
        // {
        // }
        // ;
        // {
        // }
        // ;
        // {
        // }
        // ;
        // (new_state, output_bit, )
        r16 = { r13, r14 };
        shift_register_update_4f011f2a5bb660ca = r16;
    end
endfunction

function [0:0] get_bit_4(input [3:0] a, input integer i); get_bit_4 = a[i]; endfunction
    function [3:0] set_bit_4(input [3:0] a, input integer i, input [0:0] value); set_bit_4 = value ? a | (1 << i) : a & ~(1 << i); endfunction
