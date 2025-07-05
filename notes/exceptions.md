
# interrupts

cpu interrupts are processes that have failed by iundexing with handles located in struct
**Interrupt Descriptor Table**

iretq is the instruction used for interrupts on the stack

# interrupt stack frame


## stack overflow

whenever the stack fills with functions

however, a stack pointer are not able move on, as it is stuck
on previous crashed function.

if no **guard page** pointed for stack overflow, then a double fault
occurs.
if **guard page** is not handled, then a **triple fault** occurs

to fix, use **stack switching**, which is engrained inside the CPU for x86_64 architecture
where, **stack_pointers** property allows to point to a correct stack like interrupt stack

## guard pages

area of memory to prevent stack overflows and buffer overflows

# The Interrupt Stack Table and Task State Segment

TSS holds two stack tables

PST is the other stack, which handles the privilege of where
exceptions occur
