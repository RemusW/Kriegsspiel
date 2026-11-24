# Asset ids are the SHA-256 of the image's bytes

An asset id is the lowercase hex SHA-256 of an image's bytes. Identical uploads dedupe by
themselves, and an id can never come to mean a different image, so `/assets/{id}` is
cacheable forever. A client that hashes a file with `crypto.subtle` knows the id before the
upload finishes, so it can send the command that uses the image straight after the bytes, with
no request id to correlate.
