use pokedex::{pokemon::party::Party};

use crate::pokemon::{remote::RemotePokemon, PokemonView};

pub type RemoteParty<ID, const AS: usize> =
    crate::party::PlayerParty<ID, usize, Option<RemotePokemon>, AS>;

#[derive(Debug, Clone)]
pub struct PlayerParty<ID, A: PartyIndex, P, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    pub active: [Option<A>; AS],
    pub pokemon: Party<P>,
}

/// Get the index of the pokemon in the party from this type.
pub trait PartyIndex: From<usize> {
    fn index(&self) -> usize;
}

impl PartyIndex for usize {
    fn index(&self) -> usize {
        *self
    }
}

impl<ID, A: PartyIndex, P, const AS: usize> PlayerParty<ID, A, P, AS> {
    pub fn id(&self) -> &ID {
        &self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unknown")
    }
}

impl<ID, A: PartyIndex, P, const AS: usize> PlayerParty<ID, A, P, AS> {
    pub fn index(&self, index: usize) -> Option<usize> {
        self.active
            .get(index)
            .map(|active| active.as_ref().map(PartyIndex::index))
            .flatten()
    }

    pub fn active(&self, active: usize) -> Option<&P> {
        self.index(active)
            .map(move |index| self.pokemon.get(index))
            .flatten()
    }

    pub fn active_mut(&mut self, active: usize) -> Option<&mut P> {
        self.index(active)
            .map(move |index| self.pokemon.get_mut(index))
            .flatten()
    }

    pub fn active_contains(&self, index: usize) -> bool {
        self.active
            .iter()
            .flatten()
            .any(|active| active.index() == index)
    }

    pub fn active_iter(&self) -> impl Iterator<Item = (usize, &P)> + '_ {
        self.active
            .iter()
            .enumerate()
            .flat_map(move |(index, active)| {
                active
                    .as_ref()
                    .map(|a| self.pokemon.get(a.index()).map(|p| (index, p)))
            })
            .flatten()
    }

    pub fn remove_active(&mut self, active: usize) {
        if let Some(active) = self.active.get_mut(active) {
            *active = None;
        }
    }

    pub fn add(&mut self, index: usize, pokemon: P) {
        if self.pokemon.len() > index {
            self.pokemon[index] = pokemon;
        }
    }

    pub fn take(&mut self, active: usize) -> Option<P> {
        self.index(active)
            .map(|index| {
                if self.pokemon.len() < index {
                    Some(self.pokemon.remove(index))
                } else {
                    None
                }
            })
            .flatten()
    }
}

impl<ID, A: PartyIndex, P: PokemonView, const AS: usize> PlayerParty<ID, A, P, AS> {
    pub fn new(id: ID, name: Option<String>, pokemon: Party<P>) -> Self {
        let mut active = {
            // temporary fix for const generics not implementing Default

            let mut active: [Option<A>; AS] =
                unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

            for a in active.iter_mut() {
                *a = None;
            }

            active
        };

        let mut index = 0;
        for (i, p) in pokemon.iter().enumerate() {
            if !p.fainted() {
                active[index] = Some(i.into());
                index += 1;
                if index >= active.len() {
                    break;
                }
            }
        }

        Self {
            id,
            name,
            active,
            pokemon,
        }
    }

    pub fn remaining(&self) -> impl Iterator<Item = (usize, &P)> + '_ {
        self.pokemon.iter().enumerate().filter(move |(index, p)| !self.active_contains(*index) && !p.fainted())
    }

    pub fn all_fainted(&self) -> bool {
        !self.pokemon.iter().any(|p| !p.fainted()) || self.pokemon.is_empty()
    }

    pub fn any_inactive(&self) -> bool {
        self.pokemon
            .iter()
            .enumerate()
            .filter(|(i, ..)| !self.active_contains(*i))
            .any(|(.., pokemon)| !pokemon.fainted())
    }

    pub fn active_fainted(&self) -> Option<usize> {
        self.active_iter().find(|(.., p)| p.fainted()).map(|(i, ..)| i)
    }

    pub fn needs_replace(&self) -> bool {
        self.any_inactive() && self.active.iter().any(Option::is_none)
    }

    pub fn replace(&mut self, active: usize, new: Option<usize>) {
        if let Some(a) = self.active.get_mut(active) {
            *a = new.map(Into::into);
        }
    }
}

use serde::{de::DeserializeOwned, Serialize};
use serde_big_array::BigArray;

#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, ID, A: DeserializeOwned + Serialize + PartyIndex, P, const AS: usize>
        _serde::Deserialize<'de> for PlayerParty<ID, A, P, AS>
    where
        ID: _serde::Deserialize<'de>,
        P: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "id" => _serde::__private::Ok(__Field::__field0),
                        "name" => _serde::__private::Ok(__Field::__field1),
                        "active" => _serde::__private::Ok(__Field::__field2),
                        "pokemon" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"id" => _serde::__private::Ok(__Field::__field0),
                        b"name" => _serde::__private::Ok(__Field::__field1),
                        b"active" => _serde::__private::Ok(__Field::__field2),
                        b"pokemon" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<
                'de,
                ID,
                A: DeserializeOwned + Serialize + PartyIndex,
                P,
                const AS: usize,
            >
            where
                ID: _serde::Deserialize<'de>,
                P: _serde::Deserialize<'de>,
            {
                marker: _serde::__private::PhantomData<PlayerParty<ID, A, P, AS>>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de, ID, A: DeserializeOwned + Serialize + PartyIndex, P, const AS: usize>
                _serde::de::Visitor<'de> for __Visitor<'de, ID, A, P, AS>
            where
                ID: _serde::Deserialize<'de>,
                P: _serde::Deserialize<'de>,
            {
                type Value = PlayerParty<ID, A, P, AS>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct PlayerParty")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match match _serde::de::SeqAccess::next_element::<ID>(&mut __seq)
                    {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct PlayerParty with 4 elements",
                            ));
                        }
                    };
                    let __field1 = match match _serde::de::SeqAccess::next_element::<Option<String>>(
                        &mut __seq,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct PlayerParty with 4 elements",
                            ));
                        }
                    };
                    let __field2 = match {
                        struct __DeserializeWith<
                            'de,
                            ID,
                            A: DeserializeOwned + Serialize + PartyIndex,
                            P,
                            const AS: usize,
                        >
                        where
                            ID: _serde::Deserialize<'de>,
                            P: _serde::Deserialize<'de>,
                        {
                            value: [Option<A>; AS],
                            phantom: _serde::__private::PhantomData<PlayerParty<ID, A, P, AS>>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<
                                'de,
                                ID,
                                A: DeserializeOwned + Serialize + PartyIndex,
                                P,
                                const AS: usize,
                            > _serde::Deserialize<'de> for __DeserializeWith<'de, ID, A, P, AS>
                        where
                            ID: _serde::Deserialize<'de>,
                            P: _serde::Deserialize<'de>,
                        {
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                            {
                                _serde::__private::Ok(__DeserializeWith {
                                    value: match BigArray::deserialize(__deserializer) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                    phantom: _serde::__private::PhantomData,
                                    lifetime: _serde::__private::PhantomData,
                                })
                            }
                        }
                        _serde::__private::Option::map(
                            match _serde::de::SeqAccess::next_element::<
                                __DeserializeWith<'de, ID, A, P, AS>,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            },
                            |__wrap| __wrap.value,
                        )
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct PlayerParty with 4 elements",
                            ));
                        }
                    };
                    let __field3 =
                        match match _serde::de::SeqAccess::next_element::<Party<P>>(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    3usize,
                                    &"struct PlayerParty with 4 elements",
                                ));
                            }
                        };
                    _serde::__private::Ok(PlayerParty {
                        id: __field0,
                        name: __field1,
                        active: __field2,
                        pokemon: __field3,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<ID> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<Option<String>> =
                        _serde::__private::None;
                    let mut __field2: _serde::__private::Option<[Option<A>; AS]> =
                        _serde::__private::None;
                    let mut __field3: _serde::__private::Option<Party<P>> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        }
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    match _serde::de::MapAccess::next_value::<ID>(&mut __map) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                    );
                                }
                                __field1 = _serde::__private::Some(
                                    match _serde::de::MapAccess::next_value::<Option<String>>(
                                        &mut __map,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "active",
                                        ),
                                    );
                                }
                                __field2 = _serde::__private::Some({
                                    struct __DeserializeWith<
                                        'de,
                                        ID,
                                        A: DeserializeOwned + Serialize + PartyIndex,
                                        P,
                                        const AS: usize,
                                    >
                                    where
                                        ID: _serde::Deserialize<'de>,
                                        P: _serde::Deserialize<'de>,
                                    {
                                        value: [Option<A>; AS],
                                        phantom: _serde::__private::PhantomData<
                                            PlayerParty<ID, A, P, AS>,
                                        >,
                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                    }
                                    impl<
                                            'de,
                                            ID,
                                            A: DeserializeOwned + Serialize + PartyIndex,
                                            P,
                                            const AS: usize,
                                        >
                                        _serde::Deserialize<'de>
                                        for __DeserializeWith<'de, ID, A, P, AS>
                                    where
                                        ID: _serde::Deserialize<'de>,
                                        P: _serde::Deserialize<'de>,
                                    {
                                        fn deserialize<__D>(
                                            __deserializer: __D,
                                        ) -> _serde::__private::Result<Self, __D::Error>
                                        where
                                            __D: _serde::Deserializer<'de>,
                                        {
                                            _serde::__private::Ok(__DeserializeWith {
                                                value: match BigArray::deserialize(__deserializer) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                                phantom: _serde::__private::PhantomData,
                                                lifetime: _serde::__private::PhantomData,
                                            })
                                        }
                                    }
                                    match _serde::de::MapAccess::next_value::<
                                        __DeserializeWith<'de, ID, A, P, AS>,
                                    >(&mut __map)
                                    {
                                        _serde::__private::Ok(__wrapper) => __wrapper.value,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                });
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "pokemon",
                                        ),
                                    );
                                }
                                __field3 = _serde::__private::Some(
                                    match _serde::de::MapAccess::next_value::<Party<P>>(&mut __map)
                                    {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                );
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => match _serde::__private::de::missing_field("id")
                        {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        },
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => {
                            match _serde::__private::de::missing_field("name") {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        }
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => {
                            return _serde::__private::Err(
                                <__A::Error as _serde::de::Error>::missing_field("active"),
                            )
                        }
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => {
                            match _serde::__private::de::missing_field("pokemon") {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        }
                    };
                    _serde::__private::Ok(PlayerParty {
                        id: __field0,
                        name: __field1,
                        active: __field2,
                        pokemon: __field3,
                    })
                }
            }
            const FIELDS: &'static [&'static str] = &["id", "name", "active", "pokemon"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "PlayerParty",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<PlayerParty<ID, A, P, AS>>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<ID, A: DeserializeOwned + Serialize + PartyIndex, P, const AS: usize> _serde::Serialize
        for PlayerParty<ID, A, P, AS>
    where
        ID: _serde::Serialize,
        P: _serde::Serialize,
    {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "PlayerParty",
                false as usize + 1 + 1 + 1 + 1,
            ) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "id", &self.id)
            {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "name",
                &self.name,
            ) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "active", {
                struct __SerializeWith<
                    '__a,
                    ID: '__a,
                    A: DeserializeOwned + Serialize + PartyIndex + '__a,
                    P: '__a,
                    const AS: usize,
                >
                where
                    ID: _serde::Serialize,
                    P: _serde::Serialize,
                {
                    values: (&'__a [Option<A>; AS],),
                    phantom: _serde::__private::PhantomData<PlayerParty<ID, A, P, AS>>,
                }
                impl<
                        '__a,
                        ID: '__a,
                        A: DeserializeOwned + Serialize + PartyIndex + '__a,
                        P: '__a,
                        const AS: usize,
                    > _serde::Serialize for __SerializeWith<'__a, ID, A, P, AS>
                where
                    ID: _serde::Serialize,
                    P: _serde::Serialize,
                {
                    fn serialize<__S>(
                        &self,
                        __s: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                    {
                        BigArray::serialize(self.values.0, __s)
                    }
                }
                &__SerializeWith {
                    values: (&self.active,),
                    phantom: _serde::__private::PhantomData::<PlayerParty<ID, A, P, AS>>,
                }
            }) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "pokemon",
                &self.pokemon,
            ) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
