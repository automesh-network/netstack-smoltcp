#!/bin/bash
#__author__: cavivie

DEFAULT_TUN_NAME="utun8"

function do_route() {
    local route_op="${1}"
    local tun_name="${2:-$DEFAULT_TUN_NAME}"
    ip route ${route_op} 0.0.0.0/1 dev ${tun_name}
    ip route ${route_op} 128.0.0.0/1 dev ${tun_name}
}

function usage(){
    echo "Usage:
    route add    add tun routes to system route table
    route del    delete routes from system route table
    route help   display all usages of the shell script"
}

# START MAIN-OPTIONS
case $1 in
    add) do_route add $2;;
    del) do_route delete $2;;
    *) usage ;;
esac
# END MAIN-OPTIONS
