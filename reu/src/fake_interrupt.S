.global __enable_mi
.global __disable_mi

.text
__enable_mi:
    cli
    rts

__disable_mi:
    sei
    rts