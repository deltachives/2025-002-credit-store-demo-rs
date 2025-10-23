# Demo

Hello! Welcome to the demo! This is a little program that demonstrates event sourcing with diesel.rs.

To get started,

You will need the diesel CLI. Find its installation instructions from [diesel.rs installing-diesel-cli](https://diesel.rs/guides/getting-started.html#installing-diesel-cli).

You will also need python3 and pip. Refer to their [installation instructions](https://pip.pypa.io/en/stable/installation/) if you don't have them.

then run

```sh
git clone https://github.com/deltachives/2025-002-credit-store-demo-rs.git
cd 2025-002-credit-store-demo-rs
python3 -m pip install -r requirements.txt
source ./.env; mkdir -p data; diesel migration run && python3 scripts/diesel-postprocess.py
cargo run --bin demo
```

There is autocomplete in the demo shell that can be used with `<TAB>`. You can explore all the commands by hitting it multiple times.

helptree is a command that can show all the current commands also:

```
| helptree
Normal commands
├── info
│   └── version
└── coins
    ├── users
    │   ├── add
    │   └── delete
    ├── income
    ├── expense
    ├── show
    │   ├── wallet
    │   ├── records
    │   └── partial
    │       ├── wallet
    │       └── records
    ├── toggle
    │   ├── id
    │   └── desc
    ├── ls
    ├── span
    │   ├── push
    │   └── pop
    ├── switch
    └── reset
        ├── soft
        ├── hard
        └── actually


Builtins
├── help
├── helptree
├── exit
└── history
```

You can run `help` on a specific leaf command (which is what executes).

For example:

```
| help coins users add
└─ coins -
   └─ users -
      └─ add - Add a new user to the current coin store frame with 0 coins
         └─ Called with no args
```

# Basic Usage

We start in a default span frame of (1, 1). Think of it as a coordinate for the current coin store history we're in.

We can add users with `coins users add` and also delete them with `coins users delete`. This just creates corresponding insert and delete events.

Once we have some users, we can give them income with `coins income` or expenses with `coins expense`.

We can view our wallets with `coins show wallet` which shows the total coins for each user, or `coins show records` to see the transactions.

# Deactivating Events

We can use `coins toggle id` and `coins toggle desc` to toggle events by id or by a pattern in their description.

This allows us to deactivate them so that they do not count towards the total. The results can be viewed with `coins show partial wallet` and `coins show partial records`.

# Resets

`coins reset soft` create a new frame within the same span, so all that was added in our frame will be gone, but changes prior to branching are preserved.

`coins reset hard` applies this to span 1, as there is no span lower than this, it inherits no events.

`coins reset actually` will really delete all events! Notice that we could simulate deletion without actually deleting with soft/hard resets.

TODO: Currently this soft reset behavior is buggy because events are inherited from all lower span frames, not accounting for reset.

# History Branching

All events in lower span frames are preserved in upper ones.

Use `coins ls` to see where you are in the history branches, and `coins switch` to switch to a branch of your choice.

`coins span push` will bump us a span and create latest frame there to inherit events from the lower span.

`coin span pop` will bring us back to the original branch.