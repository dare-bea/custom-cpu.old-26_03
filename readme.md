# CPU Architecture "2"

## Registers

### General Purpose

The following 8 registers are 8-bit general purpose registers (GPRs):
- H, A - Accumulator
- B, C - Loop Counter
- X, L - Memory Pointer
- M, N - Data

They can be combined to become a single 16-bit register:
- H + A = HA
- B + C = BC
- X + L = XL
- M + N = MN

### System Registers

These registers are used to run the computer. All of them of 16-bit.

- PC - Program Counter
- SP - Stack Pointer
- FL - Flags

### Encoding

Registers are encoded using 4 bits.
The first bit represents the size of the register.

0.  H
1.  A
2.  B
3.  C
4.  X
5.  L
6.  M
7.  N
8.  HA
9.  BC
10. XL
11. MN
12. _unused_
13. SP
14. PC
15. FL

### Flags

The FL register stores flags. Each bit represents a different flag:
0. ZF - Zero
1. CF - Carry
2. SF - Sign
3. OF - Overflow
4. HF - Half carry

## Endianness

The CPU uses big-endian: more significant bytes are at lower addresses.
The first register of a pair (i.e. H, B, X, M) is more significant than the
second register.

## Addressing Modes

```
movb %srcb, %dstb
ldb $immb, %dstb
ldb addr, %dstb
ldb (%srcw), %dstb
ldb off(%srcw), %dstb
stob %srcb, addr
stob %srcb, (%dstw)
stob %srcb, off(%dstw)
negb %dstb
movw %srcw, %dstw
ldw $immw, %dstw
ldw addr, %dstw
ldw (%srcw), %dstw
ldw off(%srcw), %dstw
stow %srcw, addr
stow %srcw, (%dstw)
stow %srcw, off(%dstw)
negw %dstw
jmp addr
jmp (%dstw)
jmp off(%dstw)
jmp off(%pc)
jcc addr
jcc (%dstw)
jcc off(%dstw)
jcc off(%pc)
```

`*` must be set to `0` when encoding.

```
................... - oooooooo / aa sss ddd / hhhhhhhh / llllllll
ldib $0x12, %b      - ******00 / 01 *** 010 / 00010010
movb %a, %b         - ******00 / 10 001 010
ldib $0, %b         - ******00 / 11 *** 010
ldb  0x12(%xl), %b  - ******01 / 00 010 010 / 00010010
ldb  0x1234, %b     - ******01 / 01 *** 010 / 00010011 / 00110100
stob %a, 0x12(%xl)  - ******01 / 10 010 001 / 00010010
stob %a, 0x1234     - ******01 / 11 *** 001 / 00010011 / 00110100
lea  0x12(%xl), %bc - ******10 / 00 010 001 / 00010010
ldiw $0x1234, %bc   - ******10 / 01 *** 001 / 00010010 / 00110100
movw %ha, %bc       - ******10 / 10 000 001
ldiw $0, %bc        - ******10 / 11 *** 001
ldw  0x12(%xl), %bc - ******11 / 00 010 001 / 00010010
ldw  0x1234, %bc    - ******11 / 01 *** 001 / 00010011 / 00110100
stow %ha, 0x12(%xl) - ******11 / 10 010 000 / 00010010
stow %ha, 0x1234    - ******11 / 11 *** 000 / 00010011 / 00110100

................... - oooooooo / cccc a rrr / hhhhhhhh / llllllll
jcc 0xabcd          - *******1 / xxxx 0 *** / 10101011 / 11001101
jcc 0x12(%xl)       - *******1 / xxxx 1 010 / 00010010
jcc 0x12(%pc)       - *******1 / xxxx 1 110 / 00010010
jmp 0xabcd          - *******0 / **** 0 *** / 10101011 / 11001101
jmp 0x12(%xl)       - *******0 / **** 1 010 / 00010010
jmp 0x12(%pc)       - *******0 / **** 1 110 / 00010010

................... - oooooooo / aa sss ddd / hhhhhhhh / llllllll
add %a, %b          - *****000 / 00 001 010
add $0x12, %b       - *****000 / 01 *** ddd / 00010010
add %ha, %bc        - *****000 / 10 000 001
add $0x1234, %bc    - *****000 / 11 *** ddd / 00010010 / 11001101
sub %a, %b          - *****001 / 00 001 010
sub $0x12, %b       - *****001 / 01 *** ddd / 00010010
sub %ha, %bc        - *****001 / 10 000 001
sub $0x1234, %bc    - *****001 / 11 *** ddd / 00010010 / 11001101
shl %a, %b          - *****010 / 00 001 010
shl $0x12, %b       - *****010 / 01 *** ddd / 00010010
shl %ha, %bc        - *****010 / 10 000 001
shl $0x1234, %bc    - *****010 / 11 *** ddd / 00010010 / 11001101
shr %a, %b          - *****011 / 00 001 010
shr $0x12, %b       - *****011 / 01 *** ddd / 00010010
shr %ha, %bc        - *****011 / 10 000 001
shr $0x1234, %bc    - *****011 / 11 *** ddd / 00010010 / 11001101
xor %a, %b          - *****101 / 00 001 010
xor $0x12, %b       - *****101 / 01 *** ddd / 00010010
xor %ha, %bc        - *****101 / 10 000 001
xor $0x1234, %bc    - *****101 / 11 *** ddd / 00010010 / 11001101
and %a, %b          - *****110 / 00 001 010
and $0x12, %b       - *****110 / 01 *** ddd / 00010010
and %ha, %bc        - *****110 / 10 000 001
and $0x1234, %bc    - *****110 / 11 *** ddd / 00010010 / 11001101
or  %a, %b          - *****111 / 00 001 010
or  $0x12, %b       - *****111 / 01 *** ddd / 00010010
or  %ha, %bc        - *****111 / 10 000 001
or  $0x1234, %bc    - *****111 / 11 *** ddd / 00010010 / 11001101
```


0. movb %srcb, %dstb
1. ldb $immb, %dstb
2. ldb addr, %dstb
3. ldb (%srcw), %dstb
3. ldb off(%srcw), %dstb
4. stob %srcb, addr
5. stob %srcb, (%dstw)
6. stob %srcb, off(%dstw)
7. negb %dstb
8. movw %srcw, %dstw
9. ldw $immw, %dstw
A. ldw addr, %dstw
B. ldw (%srcw), %dstw
B. ldw off(%srcw), %dstw
C. stow %srcw, addr
D. stow %srcw, (%dstw)
E. stow %srcw, off(%dstw)
F. negw %dstw

0. jmp addr
1. jmp (%dstw)
1. jmp off(%dstw)
1. jmp off(%pc)

0. jcc addr
1. jcc (%dstw)
1. jcc off(%dstw)
1. jcc off(%pc)
