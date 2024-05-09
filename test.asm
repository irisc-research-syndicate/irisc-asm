lbl setup
    set64 r5, {{ r5 }}
    set64 r6, {{ r6 }}

lbl test
    addi r7, zero, 0
    addi r8, zero, 1

lbl taken
    #st.d r0, r4, r5, 0x08, 0x2
    #st.d r0, r4, r6, 0x0c, 0x2
    #st.d r0, r4, r8, 0x10, 0x2
    #st.q r0, r4, r7, 0x18, 0x0
    #ret.d r0, r0, r0