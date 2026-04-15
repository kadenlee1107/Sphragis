#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_PROPERTY_DESCRIPTOR_OBJECT_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_PROPERTY_DESCRIPTOR_OBJECT_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=19&c=1
TNode<PropertyDescriptorObject> Cast_PropertyDescriptorObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=54&c=1
TNode<BoolT> IsCompatiblePropertyDescriptor_0(compiler::CodeAssemblerState* state_, TNode<BoolT> p__extensible, TNode<PropertyDescriptorObject> p_newDesc, TNode<PropertyDescriptorObject> p_current);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=100&c=1
TNode<BoolT> IsCompatiblePropertyDescriptor_1(compiler::CodeAssemblerState* state_, TNode<BoolT> p_extensible, TNode<Union<PropertyDescriptorObject, Undefined>> p_newDesc, TNode<PropertyDescriptorObject> p_current);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=115&c=1
TNode<BoolT> IsCompatiblePropertyDescriptor_2(compiler::CodeAssemblerState* state_, TNode<BoolT> p_extensible, TNode<Union<PropertyDescriptorObject, Undefined>> p_newDesc, TNode<Union<PropertyDescriptorObject, Undefined>> p_current);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=132&c=1
void CompletePropertyDescriptor_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_desc);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=48&c=3
TNode<Smi> LoadPropertyDescriptorObjectFlags_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=48&c=3
void StorePropertyDescriptorObjectFlags_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o, TNode<Smi> p_v);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=49&c=3
TNode<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> LoadPropertyDescriptorObjectValue_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=49&c=3
void StorePropertyDescriptorObjectValue_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o, TNode<Union<BigInt, Boolean, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=50&c=3
TNode<Union<BigInt, Boolean, FunctionTemplateInfo, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> LoadPropertyDescriptorObjectGet_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=50&c=3
void StorePropertyDescriptorObjectGet_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o, TNode<Union<BigInt, Boolean, FunctionTemplateInfo, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=51&c=3
TNode<Union<BigInt, Boolean, FunctionTemplateInfo, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> LoadPropertyDescriptorObjectSet_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=51&c=3
void StorePropertyDescriptorObjectSet_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_o, TNode<Union<BigInt, Boolean, FunctionTemplateInfo, HeapNumber, JSReceiver, Null, Smi, String, Symbol, TheHole, Undefined>> p_v);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=21&c=3
TNode<BoolT> Method_PropertyDescriptorObject_IsDataDescriptor_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_this);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=25&c=3
TNode<BoolT> Method_PropertyDescriptorObject_IsAccessorDescriptor_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_this);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=29&c=3
TNode<BoolT> Method_PropertyDescriptorObject_IsGenericDescriptor_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_this);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=36&c=3
TNode<BoolT> Method_PropertyDescriptorObject_IsEmptyOrEquivalentTo_0(compiler::CodeAssemblerState* state_, TNode<PropertyDescriptorObject> p_this, TNode<PropertyDescriptorObject> p_current);

// https://crsrc.org/c/v8/src/objects/property-descriptor-object.tq?l=19&c=1
TNode<PropertyDescriptorObject> DownCastForTorqueClass_PropertyDescriptorObject_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_PROPERTY_DESCRIPTOR_OBJECT_TQ_CSA_H_
