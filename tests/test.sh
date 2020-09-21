#!/bin/bash

if [ -t 1 ]; then
    echo "stdout is tty"
else
    echo "stdout is not tty"
fi

if [ -t 2 ]; then
    echo "stderr is tty" >&2
else
    echo "stderr is not tty" >&2
fi

exit 6
