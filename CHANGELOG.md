# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

##  (2023-02-12)


### ⚠ BREAKING CHANGES

* remove SliceBit trait
* merge BitSlice and BitSliceMut
* change trait method names to avoid collisions
* Slicing now only works for types with an exact size iterator

### Features

* **bit_slice:** more partial eq impls ([55f8202](https://github.com/Skadic/succinct_neo/commits/55f8202581f03c5f5cfc9a5e3e1ec7be80eb7f26))
* **bit_slice:** split_at ([0521185](https://github.com/Skadic/succinct_neo/commits/052118501bc66f8d8affd82764321995766c93e2))
* **bit_vec:** basic bit vector ([d09fb0d](https://github.com/Skadic/succinct_neo/commits/d09fb0d03415399582c2b43293954a684b4efcbc))
* **bs:** use a bitslice as a backing for bit_vec ([59c5c24](https://github.com/Skadic/succinct_neo/commits/59c5c246ef1e42855e7e47d8de3f1034aa9fad33))
* **bv:** add rudimentary bit slices ([2df49a8](https://github.com/Skadic/succinct_neo/commits/2df49a8dfc7bd902f48c9837be1dc4870cc70e9b))
* **int_vec:** add get_unchecked ([118c010](https://github.com/Skadic/succinct_neo/commits/118c010b1c2033ef2ae608f50bf42497beae366c))
* **int_vec:** basic int vector ([0ed5090](https://github.com/Skadic/succinct_neo/commits/0ed5090b097c79dcd755378c94bb5c3b44c53478))
* **int_vec:** extract intvec functions to trait ([d851051](https://github.com/Skadic/succinct_neo/commits/d8510516cdb4d562ecd391a7f02ba74628448029))
* **iv:** implement exact size iterator ([5e6c1a0](https://github.com/Skadic/succinct_neo/commits/5e6c1a0f51aa682086e99c3e8140955da0e4b711))
* **rs:** add select strategies for select on flatpopcount ([71cfe0f](https://github.com/Skadic/succinct_neo/commits/71cfe0fc9ac90babb4738e1c995a374b1b2ffb67))
* **rs:** add select strategies for select on flatpopcount ([7a7c366](https://github.com/Skadic/succinct_neo/commits/7a7c3668b804e818aabf721f4ac792eb8159ad3a))
* **rs:** flat popcount rank ([3a0aa52](https://github.com/Skadic/succinct_neo/commits/3a0aa5270adb3cea3205dfa11ed51ebedd2f8f21))
* **rs:** flat popcount rank ([69c827f](https://github.com/Skadic/succinct_neo/commits/69c827f597ee801266c3c64359042b1cc355bc53))
* **rs:** select support for flatpopcount ([1d38aa9](https://github.com/Skadic/succinct_neo/commits/1d38aa9c32727763312b070171a77c856cb4b604))
* **xtask:** add json coverage type ([e9e55b8](https://github.com/Skadic/succinct_neo/commits/e9e55b81f66d8135bfae59c1be70d462f80dc778))


### Bug Fixes

* add missing #[cfg(test)] ([6aefa80](https://github.com/Skadic/succinct_neo/commits/6aefa80ae3921a0b88b320eb05a76f98d869fee7))
* **bit_vec:** missing RangeToInclusive impl ([eceb02d](https://github.com/Skadic/succinct_neo/commits/eceb02d002f00c1ad05d9af9eed191cacce0075b))
* **bs:** incorrect start pos when splitting ([2392a38](https://github.com/Skadic/succinct_neo/commits/2392a38fc9688b8612e958675162c409906a666c))
* **bv:** missing raw function ([8fec22f](https://github.com/Skadic/succinct_neo/commits/8fec22fb7ae7e505468f2e0ce3fc7a19f4f8e433))
* doc tests ([c0a6e89](https://github.com/Skadic/succinct_neo/commits/c0a6e89e1cc35b00e6dedc7c4691551e433392fe))


* refactor!(bit_slice): use range bounds in slicing impls ([2cd5f3d](https://github.com/Skadic/succinct_neo/commits/2cd5f3d585fc3225e388ca5b1fab2ded051ea035))
* change trait method names to avoid collisions ([6948786](https://github.com/Skadic/succinct_neo/commits/6948786f3b317c8105a0096618c590c65d7ae595))
* merge BitSlice and BitSliceMut ([047775f](https://github.com/Skadic/succinct_neo/commits/047775fb41d2f878b9bdc30855327060fad1ffaa))
* remove SliceBit trait ([307d8b0](https://github.com/Skadic/succinct_neo/commits/307d8b05de8d66c3587cb72f5c3f0bf72a87f72f))
