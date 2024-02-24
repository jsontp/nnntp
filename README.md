# `jsontp/nnntp`
> Newer Network News Transfer Protocol
- this repo is an example of how `jsontp` can be used to build custom, predictable protocols
- the project contains `server`, a binary NNNTP server, complete with SQLite3 database usage
- it also has `client`, a library NNNTP client, with a simple interface.
- it currently implements, users, posting to newsgroups, and listing the posts of newsgroups, and it will eventually support comments on existing posts