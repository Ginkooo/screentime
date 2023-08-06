# screentime
A CLI screentime monitoring tool. Shows how much time are you using certain apps.

### Usage

1. run `screentime -d` in the background
2. `curl http:://localhost:8465/inlinehms` to get program usage times in a inline format.

Or:

`screentime hms 2023-08-30` to get a terminal output like:
```
desktop:  00:00:03
alacritty: ....... 00:07:44
firefox: ...................... 00:22:35
vim: ......................................... 00:41:00
```
