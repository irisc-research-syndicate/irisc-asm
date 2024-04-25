lbl setup
    set0 r5, r0, {{ r5.__rshift__(48).__and__(0xffff) }}
    set1 r5, r5, {{ r5.__rshift__(32).__and__(0xffff) }}
    set2 r5, r5, {{ r5.__rshift__(16).__and__(0xffff) }}
    set3 r5, r5, {{ r5.__rshift__(0).__and__(0xffff) }}
    
    set0 r6, r0, {{ r6.__rshift__(48).__and__(0xffff) }}
    set1 r6, r6, {{ r6.__rshift__(32).__and__(0xffff) }}
    set2 r6, r6, {{ r6.__rshift__(16).__and__(0xffff) }}
    set3 r6, r6, {{ r6.__rshift__(0).__and__(0xffff) }}

lbl test
    addi r7, r0, 0
    addi r8, r0, 1
    subs r0, r0, r0
    alu.r {{ funct }}, r7, r5, r6
    b.f {{ jmpop }}, r0, taken
    addi r8, r0, 0

lbl taken
    st.d r0, r4, r5, 0x08, 0x2
    st.d r0, r4, r6, 0x0c, 0x2
    st.d r0, r4, r8, 0x10, 0x2
    st.q r0, r4, r7, 0x18, 0x0
    ret.d r0, r0, r0 
