# Cài Visual Studio Build Tools (link.exe) trên Windows

Cargo cần `link.exe` để build engine-rust. File này nằm trong `setup/windows/vs_BuildTools.exe`.

Kiểm tra checksum (PowerShell):

```powershell
(Get-FileHash .\setup\windows\vs_BuildTools.exe -Algorithm SHA256).Hash.ToLower()
# phải ra: 8aaf70d164808d313ee6226af5d1e3b2db43c60be551bfeb2b70ce139d792c37
```

Cài (chạy PowerShell **Administrator**, cần internet, mất vài GB):

```powershell
.\setup\windows\vs_BuildTools.exe --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --norestart --wait
```

Sau khi cài, mở terminal mới và kiểm tra:

```powershell
where.exe link
```
