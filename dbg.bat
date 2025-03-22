@echo off
:: based on: https://stackoverflow.com/a/43385161

:start
plink -serial %1
timeout /t 2
goto start
