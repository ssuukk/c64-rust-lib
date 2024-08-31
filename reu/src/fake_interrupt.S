.global __fake_interrupt

.text
__fake_interrupt:
    inc 53280
    rti
