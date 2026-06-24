# Framelink-rs

`no_std` UART framing library for embedded Rust.

Implements COBS framing + CRC-16 error detection + ACK/NAK 
retransmit over any serial transport. Designed to run on 
bare-metal targets (RP2040, STM32) via embedded-hal.

Includes a host-side in-process emulator with deterministic 
error injection for CI testing without hardware.