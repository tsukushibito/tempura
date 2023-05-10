@echo off
setlocal enabledelayedexpansion
pushd %~dp0

for /r . %%f in (*.*) do (
    if "%%~xf"==".vert" (
        set files=!files! %%~ff
    ) else if "%%~xf"==".frag" (
        set files=!files! %%~ff
    )
)

rem echo %files%
glslc -c %files%