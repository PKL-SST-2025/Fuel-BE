# Fuel-BE

### Git Workflow Project

1. Setiap fitur dikerjakan di branch terpisah, contoh: `feat/user-table`, `feat/auth`, dll.
2. Setelah fitur selesai, push ke branch fitur masing-masing.
3. Buat Pull Request (PR) ke branch `development` untuk menggabungkan semua fitur.
4. Setelah semua fitur stabil di `development`, baru merge ke branch `main` untuk produksi/deploy.
5. Branch `main` hanya berisi kode yang sudah benar-benar siap produksi.

### Contoh Alur
- Kerjakan fitur di branch: `feat/nama-fitur`
- Push ke remote: `git push origin feat/nama-fitur`
- PR ke `development`
- Setelah semua fitur stabil, PR dari `development` ke `main`

---

## Dokumentasi API Auth (Postman)

### Register & User Management
1. **GET Semua User**
   - Method: GET
   - URL: `http://127.0.0.1:3000/users`
   - Klik Send

2. **GET User by ID**
   - Method: GET
   - URL: `http://127.0.0.1:3000/user/<id_user>`
   - Ganti `<id_user>` dengan UUID user yang ingin dilihat

3. **POST Register**
   - Method: POST
   - URL: `http://127.0.0.1:3000/register`
   - Body: raw JSON sesuai kebutuhan register

4. **UPDATE User by ID**
   - Method: PUT
   - URL: `http://127.0.0.1:3000/user/<id_user>`
   - Body: raw JSON, isi seperti register tapi tanpa password

5. **DELETE User by ID**
   - Method: DELETE
   - URL: `http://127.0.0.1:3000/user/<id_user>`
   - Klik Send

### Auth (Login)
1. **LOGIN**
   - Method: POST
   - URL: `http://127.0.0.1:3000/login`
   - Body (raw JSON):
     ```json
     {
       "email": "emailkamu@example.com",
       "password": "passwordkamu"
     }
     ```
   - Klik Send
   - Response: Data user (dan token jika nanti diaktifkan)

### Auth (Forgot Password)
1. **FORGOT PASSWORD**
   - Method: POST
   - URL: `http://127.0.0.1:3000/forgot_password`
   - Body (raw JSON):
     ```json
     {
       "email": "emailkamu@example.com",
       "new_password": "passwordbaru"
     }
     ```
   - Klik Send
   - Response: Status OK jika berhasil reset

---

## Dokumentasi API Service/Layanan

### Service Management
1. **GET Semua Service**
   - Method: `GET`
   - URL: `http://127.0.0.1:3000/services`
   - Response: Daftar semua layanan

2. **GET Service by ID**
   - Method: `GET`
   - URL: `http://127.0.0.1:3000/services/<id_service>`
   - Ganti `<id_service>` dengan UUID service yang ingin dilihat

3. **POST Buat Service Baru**
   - Method: `POST`
   - URL: `http://127.0.0.1:3000/services`
   - Body (raw JSON):
     ```json
     {
       "nama": "Cuci Mobil",
       "icon_url": "https://example.com/wash-icon.png"
     }
     ```
   - Response: Data service yang baru dibuat

4. **UPDATE Service**
   - Method: `PUT`
   - URL: `http://127.0.0.1:3000/services/<id_service>`
   - Body (raw JSON):
     ```json
     {
       "nama": "Cuci Mobil Premium",
       "icon_url": "https://example.com/premium-wash.png"
     }
     ```
   - Response: Data service yang sudah diupdate

5. **DELETE Service**
   - Method: `DELETE`
   - URL: `http://127.0.0.1:3000/services/<id_service>`
   - Response: 204 No Content jika berhasil

---

> Silakan gunakan Postman untuk mencoba endpoint di atas. Ganti parameter sesuai kebutuhan.