# DNS in a weekend

Following https://implement-dns.wizardzines.com/

## Material I found useful

* https://datatracker.ietf.org/doc/html/rfc1035
* https://veykril.github.io/tlborm/introduction.html

## TODO

This is of course very far from complete, but these are the things I want to
look into in the future.

- [ ] support ipv6
- [ ] properly handle CNAMEs in `resolve`
- [X] split code into lib and binary
- [X] make this a DNS server

## Testing

### Server

Pretty straight forward when using `dig`. Because we do not support EDNS, dig
needs to be instructed not to send a EDNS query with `+noedns` like this:

    $ dig @127.0.0.1 +noedns google.com IN A
