# ManukaZeny

A bitzeny miner (cpuminer wrapper) written in Rust

# Details(Qiita)

https://qiita.com/miyagaw61/items/f1a0aa265d3e41661914

# Install

```
cargo install --git https://github.com/miyagaw61/ManukaZeny
```

# Usage

### Prepare Miner

https://github.com/macchky/cpuminer

```
export PATH=$PATH:/path/to/cpuminer
```

### Export Slack Config

```
export RUSGIT_SLACK_URL=[slack-url]
export RUSGIT_SLACK_CHANNEL=[slack-channel]
```

### Prepare Json File


example:

```
{ "addresses": ["ABC", "IJK", "XYZ"] }
```

### Execute

```
manukazeny start [json-file]
```

### Stop Mining

```
manukazeny stop
```
