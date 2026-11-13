# Bộ cài đặt Windows (đã kiểm checksum)

File trong thư mục này lấy từ nguồn chính thức. `SHA256SUMS` là checksum của từng file.

| File | Dùng cho | Nguồn |
|---|---|---|
| `vs_BuildTools.exe` | `link.exe` cho cargo | aka.ms/vs/17/release |
| `node-v22.23.5-x64.msi` | Node.js 22 LTS (yêu cầu ≥ 20.19) | nodejs.org |
| `yt-dlp.exe` | tải nguồn video | github.com/yt-dlp/yt-dlp |
| `ffmpeg-release-essentials.zip.part-00`, `part-01` | ffmpeg + ffprobe (zip gốc 114 MB, tách để nhỏ hơn 100 MB) | gyan.dev |

## 1. Visual Studio Build Tools (PowerShell Administrator)

```powershell
.\setup\windows\vs_BuildTools.exe --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --norestart --wait
```

Mở terminal mới, chạy `where.exe link` để kiểm tra.

## 2. Node.js 22

Chạy `node-v22.23.5-x64.msi` (double-click, hoặc `msiexec /i setup\windows\node-v22.23.5-x64.msi`). Mở terminal mới, chạy `node -v` (phải ≥ v20.19).

## 3. yt-dlp

```powershell
New-Item -ItemType Directory -Force D:\tools\bin | Out-Null
Copy-Item .\setup\windows\yt-dlp.exe D:\tools\bin\
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";D:\tools\bin", "User")
```

## 4. ffmpeg + ffprobe

Ghép hai phần thành zip, rồi giải nén:

```powershell
cmd /c copy /b setup\windows\ffmpeg-release-essentials.zip.part-00+setup\windows\ffmpeg-release-essentials.zip.part-01 setup\windows\ffmpeg-release-essentials.zip
Expand-Archive setup\windows\ffmpeg-release-essentials.zip -DestinationPath D:\tools\ffmpeg
$bin = (Get-ChildItem D:\tools\ffmpeg -Recurse -Filter ffmpeg.exe | Select-Object -First 1).DirectoryName
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$bin", "User")
```

Kiểm tra checksum zip ghép: `(Get-FileHash setup\windows\ffmpeg-release-essentials.zip -Algorithm SHA256).Hash` phải ra `60F467265B1E312373DBCD92200C2618A74850F98D3D078E94296BB3FA2047BA`. Xoá zip sau khi giải nén nếu muốn.
