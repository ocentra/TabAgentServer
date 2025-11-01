# Windows Build Setup

## libclang.dll Detection

`libmdbx-sys` (a dependency of `storage`) requires `libclang.dll` to be found during build.

**Quick Setup:**

```powershell
# Run this to auto-detect and set LIBCLANG_PATH
.\setup_libclang.ps1
```

This script will:
1. Search common Visual Studio and LLVM installation locations
2. Set `LIBCLANG_PATH` for your current session
3. Show you how to set it permanently

**Manual Setup:**

If the script doesn't find `libclang.dll`, install one of:
- **Visual Studio 2022** with "Desktop development with C++" workload
- **LLVM** from https://github.com/llvm/llvm-project/releases

Then set:
```powershell
$env:LIBCLANG_PATH = "path\to\llvm\bin"
```

**Verify:**
```powershell
where.exe libclang.dll
```

