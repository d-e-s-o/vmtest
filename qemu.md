cat<<EOF > /tmp/init.sh
#!/bin/sh

echo SUCCESS
sleep 99999999999999
EOF

https://unix.stackexchange.com/questions/406051/running-qemu-with-a-root-file-system-directory-instead-of-disk-image

# Works w/ virtiofsd!!
qemu-system-x86_64 -smp 2 -m 4G -display none -nodefaults -enable-kvm -cpu host -chardev stdio,id=stdout,mux=on -serial chardev:stdout -mon chardev=stdout -object memory-backend-memfd,id=mem,size=4G,share=on -numa node,memdev=mem -chardev socket,id=char0,path=/tmp/virtiofsd.sock -device vhost-user-fs-pci,chardev=char0,tag=sysroot -kernel bzImage-virtiofsd-ready -append 'rootfstype=virtiofs root=sysroot rw earlyprintk=serial,0,115200 printk.devkmsg=on console=0,115200 loglevel=7 raid=noautodetect init=/tmp/init.sh'

-object memory-backend-memfd,id=mem,size=4G,share=on
-numa node,memdev=mem
-chardev socket,id=char0,path=/tmp/virtiofsd.sock
-device vhost-user-fs-pci,chardev=char0,tag=sysroot

-kernel bzImage-virtiofsd-ready -append 'rootfstype=virtiofs root=sysroot rw earlyprintk=serial,0,115200 printk.devkmsg=on console=0,115200 loglevel=7 raid=noautodetect init=/tmp/init.sh'

# Working w/ 9p:
qemu-system-x86_64 -smp 2 -m 4G -display none -nodefaults -enable-kvm -cpu host -chardev stdio,id=stdout,mux=on -serial chardev:stdout -mon chardev=stdout -virtfs local,id=root,path=/,mount_tag=/dev/root,security_model=none,multidevs=remap -kernel /home/muellerd/local/opt/vmtest/bzImage-v6.6-archlinux -append 'rootfstype=9p rootflags=trans=virtio,cache=mmap,msize=1048576 rw earlyprintk=serial,0,115200 printk.devkmsg=on console=0,115200 loglevel=7 raid=noautodetect init=/tmp/vmtest-initXqo1o.sh'


VMTEST NOT YET WORKING:
qemu-system-x86_64 -nodefaults -display none -serial mon:stdio -enable-kvm -cpu host -qmp unix:/tmp/qmp-209066.sock,server=on,wait=off -chardev socket,path=/tmp/qga-536236.sock,server=on,wait=off,id=qga0 -device virtio-serial -device virtserialport,chardev=qga0,name=org.qemu.guest_agent.0
-object memory-backend-memfd,id=mem,share=on,size=4G -numa node,memdev=mem
-chardev socket,id=char1,path=/tmp/virtiofsd-844378.sock
-device vhost-user-fs-pci,queue-size=1024,chardev=char1,tag=rootfs

-device virtio-serial -chardev socket,path=/tmp/cmdout-686897.sock,server=on,wait=off,id=cmdout -device virtserialport,chardev=cmdout,name=org.qemu.virtio_serial.0 -kernel /home/deso/local/opt/vmtest/bzImage-virtiofsd-ready -no-reboot -append rootfstype=virtiofs,root=rootfs rw earlyprintk=serial,0,115200 printk.devkmsg=on console=0,115200 loglevel=7 raid=noautodetect init=/tmp/vmtest-initOGsjJ.sh panic=-1 -virtfs local,id=shared,path=/home/deso/local/opt/vmtest,mount_tag=vmtest-shared,security_model=none,multidevs=remap -smp 2 -m 4G

