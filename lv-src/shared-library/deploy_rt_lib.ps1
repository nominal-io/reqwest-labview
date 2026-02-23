param(
    [string]$Ip = "192.168.86.39",

    [string]$SourceSo = "lib_http_rs_labview_64.so",

    [string]$RemoteUser = "admin",

    [string]$RemotePath = "/usr/local/lib"
)

$ErrorActionPreference = "Stop"

$resolvedSource = $SourceSo
if (-not [System.IO.Path]::IsPathRooted($SourceSo)) {
    $resolvedSource = Join-Path -Path $PSScriptRoot -ChildPath $SourceSo
}

if (-not (Test-Path -LiteralPath $resolvedSource)) {
    throw "Source library not found: $resolvedSource"
}

$target = "${RemoteUser}@${Ip}"
$remoteFile = "$RemotePath/$(Split-Path -Leaf $resolvedSource)"

Write-Host "Copying $resolvedSource to ${target}:${RemotePath}..."
scp "$resolvedSource" "${target}:${RemotePath}/"

Write-Host "Running ldconfig on ${target}..."
ssh "$target" "sudo /sbin/ldconfig || sudo ldconfig || /sbin/ldconfig || ldconfig"

Write-Host "Done. Deployed to ${remoteFile}"
