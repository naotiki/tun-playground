#!/bin/sh
ip addr flush dev tap0
ip link set dev tap0 master br0
