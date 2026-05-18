# Tutorial 1: Timer

# Experiment 1.2: Understanding how it works.
![Hasil Eksekusi Experiment 1-2](./image1-2.png)
Berdasarkan hasil eksekusi di atas, urutan teks yang tercetak di terminal adalah:
1. `Zhafira's Komputer: hey hey`
2. `Zhafira's Komputer: howdy!`
3. `Zhafira's Komputer: done!`

Teks `"hey hey"` muncul paling awal sebelum kode di dalam `spawner.spawn` dijalankan karena adanya beberapa hal seperti
1. Sifat Kelambatan Future (Lazy Evaluation) di mana dalam bahasa pemrograman Rust, sebuah blok asinkron 
(`async { ... }`) atau objek yang mengimplementasikan `Future` bersifat lazy. Artinya, ketika kita memanggil fungsi 
`spawner.spawn(...)`, Rust tidak akan langsung mengeksekusi baris kode yang ada di dalam blok tersebut. Fungsi `spawn`
hanya bertugas membungkus kode itu menjadi sebuah *task* dan memasukkannya ke dalam antrean tugas (`ready_queue`).

2. Eksekusi Sinkronus Lebih Utama di mana baris perintah `println!("...: hey hey");` ditulis secara terpisah di luar
blok asinkron. Karena berada di alur eksekusi utama yang bersifat synchronous, perintah ini akan langsung dieksekusi
oleh CPU secara instan saat alur program melewatinya, bahkan sebelum task asinkron di antrean sempat dilirik.

3. Peran Utama Executor di mana blok kode asinkron yang berisi `"howdy!"` dan `"done!"` baru benar-benar diproses dan
diperiksa ketika program menyentuh perintah `executor.run();` di baris paling bawah fungsi `main`. Executor inilah yang
bertugas mengambil tugas dari antrean dan menjalankannya, sehingga pesan asinkron baru muncul setelah pesan sinkronus
`"hey hey"` selesai dicetak.

### Experiment 1.3: Multiple Spawn and removing drop
![Hasil Eksekusi Experiment 1.3](./image1-3.png)

Berdasarkan hasil eksperimen ini, terdapat dua fenomena utama yang terjadi yaitu 

1. Interleaving atau Concurrency pada Multiple Spawn. Ketika kita menambahkan `spawner.spawn` kedua dan ketiga, ketiga
tugas asinkron tersebut berjalan secara konkuren. Output dari kedua tugas akan saling bergantian muncul di terminal
tergantung tugas mana yang menyelesaikan timer terlebih dahulu.

2. Adanya siklus ketergantungan deadlock antara komponen executor dan spawner. executor memproses antrean task
menggunakan loop `while let Ok(task) = self.ready_queue.recv()`. Metode `recv()` pada channel MPSC Rust secara inheren
bersifat blocking yang berarti perulangan tersebut akan terus berjalan dan terjaga untuk menunggu kiriman data baru
sampai saluran komunikasinya ditutup secara resmi. Di dalam sistem manajemen memori Rust, saluran MPSC hanya akan
menutup secara otomatis apabila seluruh objek pengirim data telah dropped dari memori. Ketika kita menghapus baris 
`drop(spawner);`, variabel `spawner` di dalam fungsi `main` akan tetap dianggap aktif oleh compiler. Kondisi aktifnya
`spawner` ini membuat saluran MPSC mengasumsikan bahwa masih ada kemungkinan task baru akan dikirimkan di masa
mendatang, sehingga executor menolak untuk menghentikan loop `recv()`. Di sisi lain, fungsi `main` juga tidak akan
pernah bisa mencapai akhir baris kode untuk menghancurkan `spawner` secara otomatis karena alur eksekusi utamanya sudah
terlanjur tertahan tanpa henti di dalam perintah `executor.run()`. Hubungan saling tunggu inilah yang mengunci jalannya
program dan membuatnya menggantung selamanya di terminal.