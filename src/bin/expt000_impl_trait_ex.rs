trait Talk {
    fn talk(&self, topic: String);
}

fn discuss(talker: &impl Talk) {
    talker.talk("Art".to_owned());
}

fn conversate<T: Talk>(talker: &T) {
    talker.talk("Food".to_owned());
}

struct Person {}

impl Talk for Person {
    fn talk(&self, topic: String) {
        println!("That search engine is your friend! Just go ask it about {topic}.")
    }
}

struct Cat {}

impl Talk for Cat {
    fn talk(&self, topic: String) {
        if topic.to_lowercase().contains("food") {
            println!("Meow!")
        } else {
            println!("???");
        }
    }
}

trait Noop {
    type Out;

    fn noop() -> Result<Self::Out, String>;
}

fn do_nothing<T: Noop>() -> T::Out {
    // Correct
    T::noop().unwrap()

    // Incorrect. Would error:
    /*
       error[E0790]: cannot call associated function on trait without specifying the corresponding `impl` type
       --> src/bin/expt000_impl_trait_ex.rs:43:15
       |
       36 |     fn noop() -> Result<Self::Out, String>;
       |     --------------------------------------- `Noop::noop` defined here
       ...
       43 |     let out = Noop::noop().unwrap();
       |               ^^^^^^^^^^^^ cannot call associated function of trait
       |
       help: use the fully-qualified path to the only available implementation
       |
       43 |     let out = <Cat as Noop>::noop().unwrap();
       |               +++++++     +
    */
    // let out = Noop::noop().unwrap();
    // out
}

impl Noop for Cat {
    type Out = ();

    fn noop() -> Result<Self::Out, String> {
        Ok(())
    }
}

fn main() {
    let person = Person {};

    let cat = Cat {};

    conversate(&person);

    conversate::<Cat>(&cat);

    discuss(&person);

    discuss(&cat);

    do_nothing::<Cat>();
}
