# <img src="https://github.com/bluesky-social/social-app/blob/f089f4578131e83cd177b7809ce0f7b75779dfdc/assets/logo.png" alt="bluesky logo" width="26"> Bluesky Trending Hashtags

This project demonstrates how to consume Bluesky's AT proto feed by building a "trending hashtags" feature.

It uses [Arroyo](https://github.com/ArroyoSystems/arroyo) for stream processing and [Dragonfly](https://github.com/dragonflydb/dragonfly) as a Redis replacement.

## Guide

There are two binaries.
The "ingester app" listens to Bluesky's [jetstream](https://github.com/bluesky-social/jetstream) AT proto feed and publishes to a NATS topic.
Hashtags then go through Arroyo's trending window pipeline to another NATS topic.
The "observer app" reads the NATS topic and writes the results to a Redis hash.

To run it, spin up the Docker environment:

```sh
$ just spin_up
```

You need to create Arroyo connection profiles, tables and a pipeline.
The easiest way to do is through Arroyo's web UI.
Also, `.bruno` directory contains Arroyo API request templates.

This pipeline reads tags from NATS (source - `bsky:tags` subject) and writing trending window results back to it (sink - `bsky:tagtrends` subject).

```sql
CREATE TABLE tags (
    value TEXT NOT NULL
) with (
    type = 'source',
    connector = 'nats',
    servers = 'nats:4222',
    subject = 'bsky:tags',
    -- 'auth.type' = 'credentials',
    -- 'auth.username' = '{{ NATS_USER }}',
    -- 'auth.password' = '{{ NATS_PASSWORD }}',
    format = 'json'
);

CREATE TABLE tagtrends (
    value TEXT NOT NULL,
    rank BIGINT NOT NULL
) with (
    type = 'sink',
    connector = 'nats',
    servers = 'nats:4222',
    subject = 'bsky:tagtrends',
    format = 'json'
);

INSERT INTO tagtrends
SELECT value, row_num FROM (
    SELECT *, ROW_NUMBER() OVER (
        PARTITION BY window
        ORDER BY count DESC) as row_num
    FROM (SELECT count(*) as count,
        value,
        hop(interval '5 seconds', interval '15 minutes') as window
            FROM tags
            group by value, window)) WHERE row_num <= 20;
```

Run the ingester and observer apps:

```sh
$ just ingester_run &
$ just observer_run &
```

Trending hashtags will be available in desired Redis hash, in this case `bsky:tagrank`:

```
$ docker run --rm -it --network host docker.dragonflydb.io/dragonflydb/dragonfly:v1.26.2 \
  redis-cli HGETALL bsky:tagrank
```
