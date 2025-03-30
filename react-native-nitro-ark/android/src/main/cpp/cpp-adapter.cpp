#include <jni.h>
#include "NitroArkOnLoad.hpp"

JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* vm, void*) {
  return margelo::nitro::nitroark::initialize(vm);
}
