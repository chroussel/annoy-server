FROM centos

ARG TOOLCHAIN=nightly

ADD capnp.sh /
ADD annoy.sh /
#RUN chmod +x /capnp.sh
#RUN chmod +x /annoy.sh
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN yum install -y cpp make autoconf gcc gcc-c++ wget
RUN /capnp.sh && /annoy.sh
RUN yum install -y centos-release-scl
RUN yum install -y llvm-toolset-7
RUN yum install -y openssl-devel
ENV PATH="/root/.cargo/bin:/opt/rh/llvm-toolset-7/root/usr/bin:/opt/rh/llvm-toolset-7/root/usr/sbin:${PATH}"