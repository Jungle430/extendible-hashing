# Extendible Hashing

![Rust](https://img.shields.io/badge/Extendible_Hashing-Rust-informational?style=flat-square&logo=rust&logoColor=white&color=2bbc8a)

```shell
 _____        _                     _  _  _      _         _   _              _       _   ____               _   
| ____|__  __| |_   ___  _ __    __| |(_)| |__  | |  ___  | | | |  __ _  ___ | |__   | | |  _ \  _   _  ___ | |_ 
|  _|  \ \/ /| __| / _ \| '_ \  / _` || || '_ \ | | / _ \ | |_| | / _` |/ __|| '_ \  | | | |_) || | | |/ __|| __|
| |___  >  < | |_ |  __/| | | || (_| || || |_) || ||  __/ |  _  || (_| |\__ \| | | | | | |  _ < | |_| |\__ \| |_ 
|_____|/_/\_\ \__| \___||_| |_| \__,_||_||_.__/ |_| \___| |_| |_| \__,_||___/|_| |_| | | |_| \_\ \__,_||___/ \__|
                                                                                     |_|                         


```

- This project is a Rust library implementing an [extendible hashing](https://en.wikipedia.org/wiki/Extendible_hashing), utilizing the extendible hashing algorithm. It employs a three-level structure, where the high-order bits of the hash code determine the position of slots in the first level. The slots in the first level point to a page, while the low-order bits of the hash code determine the position of elements within the page. A page's position points to a hash bucket. The buckets undergo splitting and merging based on the number of elements, reducing the need for hash table resizing. This helps to alleviate I/O pressure, making it particularly suitable for high I/O scenarios such as databases.

![Extendible Hashing](/static/extendible-htable-structure.svg)
