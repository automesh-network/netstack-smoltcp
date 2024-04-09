#!/bin/bash
#__author__: cavivie

DEFAULT_TUN_ADDR="10.10.10.2/24"
DEFAULT_TUN_DEST="10.10.10.1"

function do_route() {
    local route_op="${1}"
    local tun_addr="${2:-$DEFAULT_TUN_ADDR}"
    local tun_dest="${3:-$DEFAULT_TUN_DEST}"
    sudo route ${route_op} -net 1.0.0.0/8 ${tun_dest}
    sudo route ${route_op} -net 2.0.0.0/7 ${tun_dest}
    sudo route ${route_op} -net 4.0.0.0/6 ${tun_dest}
    sudo route ${route_op} -net 8.0.0.0/5 ${tun_dest}
    sudo route ${route_op} -net 16.0.0.0/4 ${tun_dest}
    sudo route ${route_op} -net 32.0.0.0/3 ${tun_dest}
    sudo route ${route_op} -net 64.0.0.0/2 ${tun_dest}
    sudo route ${route_op} -net 128.0.0.0/1 ${tun_dest}
    # tun2 do like this automatically
    sudo route ${route_op} -net ${tun_addr} ${tun_dest}
}

function usage(){
    echo "Usage:
    route add    add tun routes to system route table
    route del    delete routes from system route table
    route help   display all usages of the shell script"
}

# START MAIN-OPTIONS
case $1 in
    add) do_route add $2 $3;;
    del) do_route delete $2 $3;;
    *) usage ;;
esac
# END MAIN-OPTIONS
