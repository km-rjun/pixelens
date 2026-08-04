$ErrorActionPreference = 'Stop'

$packageName = 'pixelens'
$url = 'https://github.com/km-rjun/pixelens/releases/download/v0.1.4/pixelens-0.1.4-windows-x64.zip'
$checksum = ''  # Update with SHA256 checksum when releasing
$checksumType = 'sha256'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$installDir = Join-Path $toolsDir 'pixelens'

# Download and extract
$packageArgs = @{
    packageName   = $packageName
    unzipLocation = $toolsDir
    url           = $url
    checksum      = $checksum
    checksumType  = $checksumType
}
Install-ChocolateyZipPackage @packageArgs

# Add to PATH (user scope)
$pixelensPath = Join-Path $installDir
Install-ChocolateyPath $pixelensPath -PathType 'User'

# Register scheduled task for daemon (optional, user can run 'pixelens install' manually)
Write-Host "Pixelens installed to $pixelensPath"
Write-Host "Run 'pixelens install' to register the daemon as a scheduled task (requires admin)."
Write-Host "Run 'pixelensd.exe' to start the daemon manually."