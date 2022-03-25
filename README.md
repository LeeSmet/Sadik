# Sadik

Sadik is a proof of concept for the implementation of an eventually
consistent, leaderless, distributed data store system, based on [the
Dynamo paper][1]. It is currently mainly intended to research the
implementation and behavior of the system.

## Design

The goal is to build a modular framework, which can be customized to
some extend. To enforce this, each component lives in its own crate. The
biggest custom component will very likely be the storage layer.

Each physical node will contain 1 or more virtual nodes. The system
should be build in such a way to handle this, even though initially only
1 vnode will likely be run per physical node, until the implications on
the system are understood. In the ideal scenario, a sufficient study can
be done to see how the system behaves in cases of higher latency (multi
site deployments).

We assume that all nodes can communicate to one another directly.

## Current components

These are the components which are already known to be required. More
might be added in the future.

- Sadik-comms: Manages the connections between physical nodes. The
	actual implementation of the chosen network protocol(s) lives here,
	and only a generic interface to other nodes will be exposed
	(`AsyncRead + AsyncWrite`).
- Sadik-members: Handles actual message communication to other nodes.
	This also understands the concept of both virtual node and physical
	node, and maps between them. Other components can limit to virtual
	nodes.
- Sadik-hashing: Wrappers around hashing algorithms to implement traits
	needed by other components. Ideally implementing a new hash
	algorithm should only require implementing the trait(s) in this
	crate. The principle hash algorithm for Sadik will be [blake3][2].
- Sadik-node: Main vnode implementation. This might be split into
	multiple crates, or smaller yet significant pieces of functionality
	might be split of into their own crates.
- Sadik-storage: Definition of the storage traits as used by the vnode.
	Ideally adding a new storage backend should only require using this
	crate (and metrics) and implementing the trait(s).
- Sadik-metrics: Wrappers around prometheus library to inject those into
	other components. Also responsible for serving these metrics so they
	can be scraped.

[1]: https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf
[2]: https://github.com/BLAKE3-team/BLAKE3
