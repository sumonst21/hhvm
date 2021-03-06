<?hh <<__EntryPoint>> function main(): void {
$nonce = random_bytes(SODIUM_CRYPTO_SECRETBOX_NONCEBYTES);
$key = sodium_crypto_secretbox_keygen();

$a = sodium_crypto_secretbox('test', $nonce, $key);
$x = sodium_crypto_secretbox_open($a, $nonce, $key);
var_dump(bin2hex($x));
$y = sodium_crypto_secretbox_open("\0" . $a, $nonce, $key);
var_dump($y);

try {
    sodium_crypto_secretbox('test', substr($nonce, 1), $key);
} catch (SodiumException $ex) {
    var_dump(true);
}
}
