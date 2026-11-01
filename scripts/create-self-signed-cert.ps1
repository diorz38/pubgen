param(
    [string] $CertName = "DesignCraftLocalTest",
    [string] $OutPath = ".\designcraft_test_cert.pfx",
    [string] $Password = "DesignCraft2026!"
)

$ErrorActionPreference = 'Stop'

Write-Host "==> Creating self-signed code signing certificate..."
$cert = New-SelfSignedCertificate `
    -DnsName "DesignCraft" `
    -Type CodeSigningCert `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -FriendlyName $CertName `
    -NotAfter (Get-Date).AddYears(5)

Write-Host "==> Exporting certificate to $OutPath..."
$securePassword = ConvertTo-SecureString -String $Password -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath $OutPath -Password $securePassword | Out-Null

Write-Host "==> Success! Certificate created and exported."
Write-Host "    File: $OutPath"
Write-Host "    Password: $Password"
Write-Host ""
Write-Host "To use this in PowerShell for packaging & signing:"
Write-Host '  $env:WINDOWS_CERTIFICATE = [Convert]::ToBase64String([System.IO.File]::ReadAllBytes("designcraft_test_cert.pfx"))'
Write-Host "  `$env:WINDOWS_CERTIFICATE_PASSWORD = '$Password'"
