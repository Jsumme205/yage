use super::{System, Systems};

impl<S1, A> Systems<A> for S1
where
    S1: System<A>,
{
    type Collections = S1::Collection;

    fn run_systems(&mut self, collections: &mut Self::Collections) {
        self.run_system(collections);
    }
}

impl<S1, S2, A, B> Systems<(A, B)> for (S1, S2)
where
    S1: System<A>,
    S2: System<B>,
{
    type Collections = (S1::Collection, S2::Collection);

    fn run_systems(&mut self, collections: &mut Self::Collections) {
        let (s1, s2) = self;
        let (c1, c2) = collections;
        s1.run_system(c1);
        s2.run_system(c2);
    }
}

impl<S1, S2, S3, A, B, C> Systems<(A, B, C)> for (S1, S2, S3)
where
    S1: System<A>,
    S2: System<B>,
    S3: System<C>,
{
    type Collections = (S1::Collection, S2::Collection, S3::Collection);

    fn run_systems(&mut self, collections: &mut Self::Collections) {
        let (s1, s2, s3) = self;
        let (c1, c2, c3) = collections;
        s1.run_system(c1);
        s2.run_system(c2);
        s3.run_system(c3);
    }
}

impl<S1, S2, S3, S4, A, B, C, D> Systems<(A, B, C, D)> for (S1, S2, S3, S4)
where
    S1: System<A>,
    S2: System<B>,
    S3: System<C>,
    S4: System<D>,
{
    type Collections = (
        S1::Collection,
        S2::Collection,
        S3::Collection,
        S4::Collection,
    );

    fn run_systems(&mut self, collections: &mut Self::Collections) {
        let (s1, s2, s3, s4) = self;
        let (c1, c2, c3, c4) = collections;
        s1.run_system(c1);
        s2.run_system(c2);
        s3.run_system(c3);
        s4.run_system(c4);
    }
}
