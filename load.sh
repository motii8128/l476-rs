#!/bin/bash

echo "----------- Build and Compile ----------"

cargo build --release

echo "----------- Start Loading binary ----------"

echo "Binary Name :" $1

openocd -f openocd.cfg -c "flash_elf $1"