#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_FEEDBACK_VECTOR_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_FEEDBACK_VECTOR_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=27&c=1
TNode<ClosureFeedbackCellArray> Cast_ClosureFeedbackCellArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=33&c=1
TNode<FeedbackVector> Cast_FeedbackVector_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=47&c=1
TNode<FeedbackMetadata> Cast_FeedbackMetadata_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=29&c=9
TNode<Smi> LoadClosureFeedbackCellArrayLength_0(compiler::CodeAssemblerState* state_, TNode<ClosureFeedbackCellArray> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=30&c=3
TorqueStructSlice_FeedbackCell_MutableReference_FeedbackCell_0 FieldSliceClosureFeedbackCellArrayObjects_0(compiler::CodeAssemblerState* state_, TNode<ClosureFeedbackCellArray> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=30&c=3
TNode<FeedbackCell> LoadClosureFeedbackCellArrayObjects_0(compiler::CodeAssemblerState* state_, TNode<ClosureFeedbackCellArray> p_o, TNode<IntPtrT> p_i);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=30&c=3
void StoreClosureFeedbackCellArrayObjects_0(compiler::CodeAssemblerState* state_, TNode<ClosureFeedbackCellArray> p_o, TNode<IntPtrT> p_i, TNode<FeedbackCell> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=35&c=9
TNode<Int32T> LoadFeedbackVectorLength_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=36&c=3
TNode<Int32T> LoadFeedbackVectorInvocationCount_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=36&c=3
void StoreFeedbackVectorInvocationCount_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<Int32T> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=38&c=3
TNode<Uint8T> LoadFeedbackVectorInvocationCountBeforeStable_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=38&c=3
void StoreFeedbackVectorInvocationCountBeforeStable_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<Uint8T> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=39&c=3
TNode<Uint8T> LoadFeedbackVectorOsrState_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=39&c=3
void StoreFeedbackVectorOsrState_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<Uint8T> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=40&c=3
TNode<Uint16T> LoadFeedbackVectorFlags_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=40&c=3
void StoreFeedbackVectorFlags_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<Uint16T> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=41&c=3
TNode<SharedFunctionInfo> LoadFeedbackVectorSharedFunctionInfo_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=41&c=3
void StoreFeedbackVectorSharedFunctionInfo_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<SharedFunctionInfo> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=42&c=3
TNode<ClosureFeedbackCellArray> LoadFeedbackVectorClosureFeedbackCellArray_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=42&c=3
void StoreFeedbackVectorClosureFeedbackCellArray_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<ClosureFeedbackCellArray> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=43&c=3
TNode<FeedbackCell> LoadFeedbackVectorParentFeedbackCell_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=43&c=3
void StoreFeedbackVectorParentFeedbackCell_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<FeedbackCell> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=44&c=36
TorqueStructSlice_MaybeObject_MutableReference_MaybeObject_0 FieldSliceFeedbackVectorRawFeedbackSlots_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=44&c=36
TNode<Union<HeapObject, Smi, Weak<HeapObject>>> LoadFeedbackVectorRawFeedbackSlots_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<IntPtrT> p_i);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=44&c=36
void StoreFeedbackVectorRawFeedbackSlots_0(compiler::CodeAssemblerState* state_, TNode<FeedbackVector> p_o, TNode<IntPtrT> p_i, TNode<Union<HeapObject, Smi, Weak<HeapObject>>> p_v);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=27&c=1
TNode<ClosureFeedbackCellArray> DownCastForTorqueClass_ClosureFeedbackCellArray_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=33&c=1
TNode<FeedbackVector> DownCastForTorqueClass_FeedbackVector_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=47&c=1
TNode<FeedbackMetadata> DownCastForTorqueClass_FeedbackMetadata_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=30&c=3
TorqueStructSlice_FeedbackCell_MutableReference_FeedbackCell_0 NewMutableSlice_FeedbackCell_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset, TNode<IntPtrT> p_length);

// https://crsrc.org/c/v8/src/objects/feedback-vector.tq?l=44&c=36
TorqueStructSlice_MaybeObject_MutableReference_MaybeObject_0 NewMutableSlice_MaybeObject_0(compiler::CodeAssemblerState* state_, TNode<Union<HeapObject, TaggedIndex>> p_object, TNode<IntPtrT> p_offset, TNode<IntPtrT> p_length);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_FEEDBACK_VECTOR_TQ_CSA_H_
