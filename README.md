# [Описание задачи](https://gitverse.ru/sb-rs/cource/content/master/assignments/udp-hole.md)

- [x] Регистрируется на сервере, сообщая свой внешний IP и порт через UDP.
- [x] Получает адрес пира через HTTP-запрос.
- [x] Устанавливает прямое UDP-соединение, обходя NAT.
- [x] Обменивается текстовыми сообщениями.

![image](https://github.com/user-attachments/assets/81e87189-78e3-447d-be86-1bbeb1aa7852)

# Примечания

* Из-за странностей рандеву сервера (кеширования адресов при подключении) иногда приходится перезапускать клиент пока кеш на сервере не инвалидируется
* Было решено использовать подход без явного ожидающего клиента - `cli.client` параметр в документации. Каждый клиент как и принимает так и отправляет сообщения 
