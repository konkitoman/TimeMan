# TimeMan (cli)

[![Crates.io](https://img.shields.io/crates/v/timeman.svg)](https://crates.io/crates/timeman)

Is a time and date calculator.

## How to use

- `tm now` to get the current time, output = `Tue, 23 Apr 2024 11:40:37 +0300`
- `tm since "$(tm now)"` or `tm s "$(tm now)"` this will output a duration, output = `PT0.51623755S`
- `tm sub "$(tm now)" "Tue, 23 Apr 2024 11:40:37 +0300"` this will output a duration, output = `PT8M15S`
- `tm sub-duration "$(tm now)" PT8M15S` this will output a date and time, output = `Tue, 23 Apr 2024 11:43:38 +0300`
- `tm add-duration "$(tm now)" PT8M15S` this will output a date and time, output = `Tue, 23 Apr 2024 12:01:38 +0300`
- `tm translate -F"%+" "$(tm now)"` use this to change format or UTC offset, output = `2024-04-23T11:55:17+03:00`
- `tm help-format` to find out how to make your own format, the output will be like = `%A : Full day of the week names.` 
- `tm help-format %A` to see more info 
- `tm help-format date` to search for any thing that has date in description
- `tm help-duration` to learn the duration flags

default format is the: `%a, %d %b %Y %T %z` is the same as `date -R` or rfc-email

## How to install

This is a rust application you need to have the toolchain installed!

`cargo install timeman` or you can install from Releases

