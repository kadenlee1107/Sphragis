#include "src/objects/templates-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/templates.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/templates.tq?l=11&c=1
bool IsTemplateInfo_NonInline(Tagged<HeapObject> o) {
  return IsTemplateInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTemplateInfo<TemplateInfo, HeapObject>::TemplateInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TemplateInfoVerify(TrustedCast<TemplateInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/templates.tq?l=16&c=1
bool IsTemplateInfoWithProperties_NonInline(Tagged<HeapObject> o) {
  return IsTemplateInfoWithProperties(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedTemplateInfoWithProperties<TemplateInfoWithProperties, TemplateInfo>::TemplateInfoWithPropertiesVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::TemplateInfoWithPropertiesVerify(TrustedCast<TemplateInfoWithProperties>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/templates.tq?l=23&c=1
class TorqueGeneratedFunctionTemplateRareDataAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=26&c=3
  static constexpr int kPrototypeTemplateOffset = sizeof(StructLayout);
  static constexpr int kPrototypeTemplateOffsetEnd = kPrototypeTemplateOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=27&c=3
  static constexpr int kPrototypeProviderTemplateOffset = kPrototypeTemplateOffsetEnd + 1;
  static constexpr int kPrototypeProviderTemplateOffsetEnd = kPrototypeProviderTemplateOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=28&c=3
  static constexpr int kParentTemplateOffset = kPrototypeProviderTemplateOffsetEnd + 1;
  static constexpr int kParentTemplateOffsetEnd = kParentTemplateOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=29&c=3
  static constexpr int kNamedPropertyHandlerOffset = kParentTemplateOffsetEnd + 1;
  static constexpr int kNamedPropertyHandlerOffsetEnd = kNamedPropertyHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=30&c=3
  static constexpr int kIndexedPropertyHandlerOffset = kNamedPropertyHandlerOffsetEnd + 1;
  static constexpr int kIndexedPropertyHandlerOffsetEnd = kIndexedPropertyHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=31&c=3
  static constexpr int kInstanceTemplateOffset = kIndexedPropertyHandlerOffsetEnd + 1;
  static constexpr int kInstanceTemplateOffsetEnd = kInstanceTemplateOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=32&c=3
  static constexpr int kInstanceCallHandlerOffset = kInstanceTemplateOffsetEnd + 1;
  static constexpr int kInstanceCallHandlerOffsetEnd = kInstanceCallHandlerOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=33&c=3
  static constexpr int kAccessCheckInfoOffset = kInstanceCallHandlerOffsetEnd + 1;
  static constexpr int kAccessCheckInfoOffsetEnd = kAccessCheckInfoOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/templates.tq?l=34&c=3
  static constexpr int kCFunctionOverloadsOffset = kAccessCheckInfoOffsetEnd + 1;
  static constexpr int kCFunctionOverloadsOffsetEnd = kCFunctionOverloadsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kCFunctionOverloadsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kCFunctionOverloadsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kCFunctionOverloadsOffsetEnd + 1;
  static constexpr int kHeaderSize = kCFunctionOverloadsOffsetEnd + 1;
  static constexpr int kSize = kCFunctionOverloadsOffsetEnd + 1;

  static_assert(kPrototypeTemplateOffset == offsetof(FunctionTemplateRareData, prototype_template_),
                "Value of FunctionTemplateRareData::kPrototypeTemplateOffset defined in Torque and offset of field FunctionTemplateRareData::prototype_template in C++ do not match");
  static_assert(kPrototypeProviderTemplateOffset == offsetof(FunctionTemplateRareData, prototype_provider_template_),
                "Value of FunctionTemplateRareData::kPrototypeProviderTemplateOffset defined in Torque and offset of field FunctionTemplateRareData::prototype_provider_template in C++ do not match");
  static_assert(kParentTemplateOffset == offsetof(FunctionTemplateRareData, parent_template_),
                "Value of FunctionTemplateRareData::kParentTemplateOffset defined in Torque and offset of field FunctionTemplateRareData::parent_template in C++ do not match");
  static_assert(kNamedPropertyHandlerOffset == offsetof(FunctionTemplateRareData, named_property_handler_),
                "Value of FunctionTemplateRareData::kNamedPropertyHandlerOffset defined in Torque and offset of field FunctionTemplateRareData::named_property_handler in C++ do not match");
  static_assert(kIndexedPropertyHandlerOffset == offsetof(FunctionTemplateRareData, indexed_property_handler_),
                "Value of FunctionTemplateRareData::kIndexedPropertyHandlerOffset defined in Torque and offset of field FunctionTemplateRareData::indexed_property_handler in C++ do not match");
  static_assert(kInstanceTemplateOffset == offsetof(FunctionTemplateRareData, instance_template_),
                "Value of FunctionTemplateRareData::kInstanceTemplateOffset defined in Torque and offset of field FunctionTemplateRareData::instance_template in C++ do not match");
  static_assert(kInstanceCallHandlerOffset == offsetof(FunctionTemplateRareData, instance_call_handler_),
                "Value of FunctionTemplateRareData::kInstanceCallHandlerOffset defined in Torque and offset of field FunctionTemplateRareData::instance_call_handler in C++ do not match");
  static_assert(kAccessCheckInfoOffset == offsetof(FunctionTemplateRareData, access_check_info_),
                "Value of FunctionTemplateRareData::kAccessCheckInfoOffset defined in Torque and offset of field FunctionTemplateRareData::access_check_info in C++ do not match");
  static_assert(kCFunctionOverloadsOffset == offsetof(FunctionTemplateRareData, c_function_overloads_),
                "Value of FunctionTemplateRareData::kCFunctionOverloadsOffset defined in Torque and offset of field FunctionTemplateRareData::c_function_overloads in C++ do not match");
  static_assert(kSize == sizeof(FunctionTemplateRareData));
};

// https://crsrc.org/c/v8/src/objects/templates.tq?l=55&c=1
bool IsFunctionTemplateInfo_NonInline(Tagged<HeapObject> o) {
  return IsFunctionTemplateInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedFunctionTemplateInfo<FunctionTemplateInfo, TemplateInfoWithProperties>::FunctionTemplateInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::FunctionTemplateInfoVerify(TrustedCast<FunctionTemplateInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/templates.tq?l=115&c=1
bool IsObjectTemplateInfo_NonInline(Tagged<HeapObject> o) {
  return IsObjectTemplateInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedObjectTemplateInfo<ObjectTemplateInfo, TemplateInfoWithProperties>::ObjectTemplateInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::ObjectTemplateInfoVerify(TrustedCast<ObjectTemplateInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// https://crsrc.org/c/v8/src/objects/templates.tq?l=121&c=1
bool IsDictionaryTemplateInfo_NonInline(Tagged<HeapObject> o) {
  return IsDictionaryTemplateInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedDictionaryTemplateInfo<DictionaryTemplateInfo, TemplateInfo>::DictionaryTemplateInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::DictionaryTemplateInfoVerify(TrustedCast<DictionaryTemplateInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
