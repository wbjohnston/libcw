# Core Wars Library
Supporting functions and runtime for the game corewars

![Travis Status](https://travis-ci.org/wbjohnston/libcw.svg?branch=master)

## RedCode
#### Redcode Opcodes
|Opcode                  |Description                                          |
|:----------------------:|:----------------------------------------------------|
|`dat`                   |Data, terminates thread on execution                 |
|`mov`                   |Move, copies instruction from `src` to `target`      |
|`add`                   |Add                                                  |
|`sub`                   |Subtract                                             |
|`mul`                   |Multiply                                             |
|`div`                   |Divide                                               |
|`mod`                   |Modulo                                               |
|`jmp`                   |Unconditional jump                                   |
|`jmz`                   |Jump if zero                                         |
|`jmn`                   |Jump if not zero                                     |
|`djn`                   |Decrement and jump if not zero                       |
|`spl`                   |Split (add address to process queue)                 |
|`cmp`                   |Compare (same as `seq`)                              |
|`seq`                   |Skip if equal                                        |
|`sne`                   |Skip if not equal                                    |
|`slt`                   |Skip if less than                                    |
|`ldp`                   |Load from P-space                                    |
|`stp`                   |Save to P-space                                      |
|`nop`                   |No operation                                         |

#### Opcode Modifiers
|Mode                    |Description                                          |
|:----------------------:|:----------------------------------------------------|
|`A`                     |A-field to A-field                                   |
|`B`                     |B-field to B-field                                   |
|`AB`                    |A-field to B-field                                   |
|`BA`                    |B-field to A-field                                   |
|`F`                     |A-field to A-field AND B-field to B-field            |
|`X`                     |A-field to B-field AND B-field to A-field            |
|`I`                     |Entire Opcode is moved                               |

#### Addressing Modes
|Mode                    |Description                                          |
|:----------------------:|:----------------------------------------------------|
|`#`                     |Immediate                                            |
|`$`                     |Direct                                               |
|`*`                     |A-field indirect                                     |
|`@`                     |B-field indirect                                     |
|`{`                     |A-field indirect with predecrement                   |
|`<`                     |B-field indirect with predecrement                   |
|`}`                     |A-field indirect with postincrement                  |
|`>`                     |B-field indirect with postincrement                  |

### Example Programs
#### The Imp
```
    MOV 0, 1
```

#### The Dwarf
```
    ADD #4, 3
    MOV 2, @2
    JMP -2
    DAT #0, #0
```

## License
#### MIT
Copyright © 2017 Will Johnston

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the "Software"),
to deal in the Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included
in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

