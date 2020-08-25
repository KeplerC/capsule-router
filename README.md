# Capsule Router

A basic UDP router with Capsule framework. 

## Running the application

Building the capsule and sandbox environment by the official `Vagrant` [virtual machine](https://github.com/capsule-rs/sandbox/blob/master/Vagrantfile) and the `Docker` [sandbox](https://hub.docker.com/repository/docker/getcapsule/sandbox). 
```
host$ git clone --recursive https://github.com/capsule-rs/sandbox.git
host$ cd sandbox
host$ vagrant up
```

Then we put our `capsule-router` in the subdirectory of capsule. 
```
host$ cd sandbox/capsule
host$ git clone https://github.com/KeplerC/capsule-router
```
Then we modify `sandbox/capsule/Cargo.toml` by adding `capsule-router` to `members`. 

After setting up the code environment, we enter the virtual machine and docker by 
```
host$ vagrant ssh
vagrant$ docker run -it --rm     --privileged     --network=host     --name sandbox     --cap-add=SYS_PTRACE     --security-opt seccomp=unconfined     -v /lib/modules:/lib/modules   -v /vagrant/capsule:/capsule  -v /dev/hugepages:/dev/hugepages     getcapsule/sandbox:19.11.1-1.43 /bin/bash
```

Finally, we run example by 
```
cd capsule/capsule-router
cargo run -- -f capsule-router.toml
```
