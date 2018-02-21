package com.sun.jna

// NativeString is package private...
fun String.toJnaPointer(encoding: String) = NativeString(this, encoding).pointer
