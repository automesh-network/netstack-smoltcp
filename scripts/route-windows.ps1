#__author__: cavivie

param(
    [string]$Cmd = "help",
    [string]$TunName = "utun8",
    [string]$TunGateway = "10.10.10.1"
)

$ErrorActionPreference = "Stop"

# START MAIN-OPTIONS
switch ($Cmd) {
    "add" {
        # tun2 do like this automatically
        New-NetRoute -DestinationPrefix "0.0.0.0/1" -InterfaceAlias $TunName -NextHop "$TunGateway"
        New-NetRoute -DestinationPrefix "128.0.0.0/1" -InterfaceAlias $TunName -NextHop "$TunGateway"
    }
    "del" {
        # tun2 do like this automatically
        Get-NetRoute -DestinationPrefix "0.0.0.0/1" -InterfaceAlias $TunName | Remove-NetRoute
        Get-NetRoute -DestinationPrefix "128.0.0.0/1" -InterfaceAlias $TunName | Remove-NetRoute
    }
    default {
        Write-Host "Usage:
    route add    add tun routes to system route table
    route del    delete routes from system route table
    route help   display all usages of the shell script"
    }
}
# END MAIN-OPTIONS
