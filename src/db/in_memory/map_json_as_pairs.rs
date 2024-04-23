pub mod map_as_pairs {
    use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;
    use std::marker::PhantomData;

    pub fn serialize<K, V, M, S>(map: M, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Serialize,
        V: Serialize,
        M: IntoIterator<Item = (K, V)>,
        S: Serializer,
    {
        serializer.collect_seq(map)
    }

    pub fn deserialize<'de, K, V, M, D>(deserializer: D) -> Result<M, D::Error>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        M: Default + Extend<(K, V)>,
        D: Deserializer<'de>,
    {
        struct MapVisitor<K, V, M> {
            keys: PhantomData<K>,
            values: PhantomData<V>,
            map: PhantomData<M>,
        }

        impl<'de, K, V, M> Visitor<'de> for MapVisitor<K, V, M>
        where
            K: Deserialize<'de>,
            V: Deserialize<'de>,
            M: Default + Extend<(K, V)>,
        {
            type Value = M;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of key-value pairs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = M::default();
                while let Some((k, v)) = seq.next_element()? {
                    map.extend(Some((k, v)));
                }
                Ok(map)
            }
        }

        deserializer.deserialize_seq(MapVisitor {
            keys: PhantomData,
            values: PhantomData,
            map: PhantomData,
        })
    }
}
