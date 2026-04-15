#include "src/objects/shared-function-info-inl.h"

#include "torque-generated/class-verifiers.h"
#include "src/objects/objects-inl.h"

#include "src/objects/instance-type-inl.h"

#include "src/objects/shared-function-info.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=7&c=1
class TorqueGeneratedPreparseDataAsserts {
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=10&c=3
  static constexpr int kDataLengthOffset = sizeof(HeapObjectLayout);
  static constexpr int kDataLengthOffsetEnd = kDataLengthOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=11&c=3
  static constexpr int kChildrenLengthOffset = kDataLengthOffsetEnd + 1;
  static constexpr int kChildrenLengthOffsetEnd = kChildrenLengthOffset + kInt32Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kChildrenLengthOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kChildrenLengthOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kChildrenLengthOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kChildrenLengthOffsetEnd + 1;
  static constexpr int kHeaderSize = kChildrenLengthOffsetEnd + 1;
  static constexpr int kSize = kChildrenLengthOffsetEnd + 1;

  static_assert(kDataLengthOffset == offsetof(PreparseData, data_length_),
                "Value of PreparseData::kDataLengthOffset defined in Torque and offset of field PreparseData::data_length in C++ do not match");
  static_assert(kChildrenLengthOffset == offsetof(PreparseData, children_length_),
                "Value of PreparseData::kChildrenLengthOffset defined in Torque and offset of field PreparseData::children_length in C++ do not match");
  static_assert(kSize == sizeof(PreparseData));
};

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=14&c=1
class TorqueGeneratedInterpreterDataAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(ExposedTrustedObjectLayout);
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=16&c=3
  static constexpr int kBytecodeArrayOffset = sizeof(ExposedTrustedObjectLayout);
  static constexpr int kBytecodeArrayOffsetEnd = kBytecodeArrayOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=17&c=3
  static constexpr int kInterpreterTrampolineOffset = kBytecodeArrayOffsetEnd + 1;
  static constexpr int kInterpreterTrampolineOffsetEnd = kInterpreterTrampolineOffset + kTaggedSize - 1;
  static constexpr int kEndOfWeakFieldsOffset = kInterpreterTrampolineOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kInterpreterTrampolineOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kInterpreterTrampolineOffsetEnd + 1;
  static constexpr int kHeaderSize = kInterpreterTrampolineOffsetEnd + 1;
  static constexpr int kSize = kInterpreterTrampolineOffsetEnd + 1;

  static_assert(kBytecodeArrayOffset == offsetof(InterpreterData, bytecode_array_),
                "Value of InterpreterData::kBytecodeArrayOffset defined in Torque and offset of field InterpreterData::bytecode_array in C++ do not match");
  static_assert(kInterpreterTrampolineOffset == offsetof(InterpreterData, interpreter_trampoline_),
                "Value of InterpreterData::kInterpreterTrampolineOffset defined in Torque and offset of field InterpreterData::interpreter_trampoline in C++ do not match");
  static_assert(kSize == sizeof(InterpreterData));
};

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=63&c=1
bool IsSharedFunctionInfo_NonInline(Tagged<HeapObject> o) {
  return IsSharedFunctionInfo(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedSharedFunctionInfo<SharedFunctionInfo, HeapObject>::SharedFunctionInfoVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::SharedFunctionInfoVerify(TrustedCast<SharedFunctionInfo>(*this), isolate);
}


#endif  // VERIFY_HEAP
// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=130&c=1
class TorqueGeneratedSharedFunctionInfoWrapperAsserts {
  static constexpr int kStartOfStrongFieldsOffset = TrustedObject::kHeaderSize;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=132&c=3
  static constexpr int kSharedInfoOffset = TrustedObject::kHeaderSize;
  static constexpr int kSharedInfoOffsetEnd = kSharedInfoOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kSharedInfoOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kSharedInfoOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kSharedInfoOffsetEnd + 1;
  static constexpr int kHeaderSize = kSharedInfoOffsetEnd + 1;
  static constexpr int kSize = kSharedInfoOffsetEnd + 1;

  static_assert(kSharedInfoOffset == SharedFunctionInfoWrapper::kSharedInfoOffset,
                "Values of SharedFunctionInfoWrapper::kSharedInfoOffset defined in Torque and C++ do not match");
  static_assert(kSize == SharedFunctionInfoWrapper::kSize);
};

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=161&c=1
class TorqueGeneratedUncompiledDataAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(ExposedTrustedObjectLayout);
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=164&c=3
  static constexpr int kInferredNameOffset = sizeof(ExposedTrustedObjectLayout);
  static constexpr int kInferredNameOffsetEnd = kInferredNameOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kInferredNameOffsetEnd + 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=165&c=3
  static constexpr int kStartPositionOffset = kInferredNameOffsetEnd + 1;
  static constexpr int kStartPositionOffsetEnd = kStartPositionOffset + kInt32Size - 1;
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=166&c=3
  static constexpr int kEndPositionOffset = kStartPositionOffsetEnd + 1;
  static constexpr int kEndPositionOffsetEnd = kEndPositionOffset + kInt32Size - 1;
  static constexpr int kStartOfWeakFieldsOffset = kEndPositionOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kEndPositionOffsetEnd + 1;
  static constexpr int kHeaderSize = kEndPositionOffsetEnd + 1;

  static_assert(kInferredNameOffset == offsetof(UncompiledData, inferred_name_),
                "Value of UncompiledData::kInferredNameOffset defined in Torque and offset of field UncompiledData::inferred_name in C++ do not match");
  static_assert(kStartPositionOffset == offsetof(UncompiledData, start_position_),
                "Value of UncompiledData::kStartPositionOffset defined in Torque and offset of field UncompiledData::start_position in C++ do not match");
  static_assert(kEndPositionOffset == offsetof(UncompiledData, end_position_),
                "Value of UncompiledData::kEndPositionOffset defined in Torque and offset of field UncompiledData::end_position in C++ do not match");
};

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=169&c=1
class TorqueGeneratedUncompiledDataWithoutPreparseDataAsserts {
  static constexpr int kStartOfWeakFieldsOffset = sizeof(UncompiledData);
  static constexpr int kEndOfWeakFieldsOffset = sizeof(UncompiledData);
  static constexpr int kStartOfStrongFieldsOffset = sizeof(UncompiledData);
  static constexpr int kEndOfStrongFieldsOffset = sizeof(UncompiledData);
  static constexpr int kHeaderSize = sizeof(UncompiledData);
  static constexpr int kSize = sizeof(UncompiledData);

  static_assert(kSize == sizeof(UncompiledDataWithoutPreparseData));
};

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=172&c=1
class TorqueGeneratedUncompiledDataWithPreparseDataAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(UncompiledData);
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=174&c=3
  static constexpr int kPreparseDataOffset = sizeof(UncompiledData);
  static constexpr int kPreparseDataOffsetEnd = kPreparseDataOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kPreparseDataOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kPreparseDataOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kPreparseDataOffsetEnd + 1;
  static constexpr int kHeaderSize = kPreparseDataOffsetEnd + 1;
  static constexpr int kSize = kPreparseDataOffsetEnd + 1;

  static_assert(kPreparseDataOffset == offsetof(UncompiledDataWithPreparseData, preparse_data_),
                "Value of UncompiledDataWithPreparseData::kPreparseDataOffset defined in Torque and offset of field UncompiledDataWithPreparseData::preparse_data in C++ do not match");
  static_assert(kSize == sizeof(UncompiledDataWithPreparseData));
};

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=177&c=1
class TorqueGeneratedUncompiledDataWithoutPreparseDataWithJobAsserts {
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=180&c=3
  static constexpr int kJobOffset = sizeof(UncompiledDataWithoutPreparseData);
  static constexpr int kJobOffsetEnd = kJobOffset + kSystemPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kHeaderSize = kJobOffsetEnd + 1;
  static constexpr int kSize = kJobOffsetEnd + 1;

  static_assert(kJobOffset == offsetof(UncompiledDataWithoutPreparseDataWithJob, job_),
                "Value of UncompiledDataWithoutPreparseDataWithJob::kJobOffset defined in Torque and offset of field UncompiledDataWithoutPreparseDataWithJob::job in C++ do not match");
  static_assert(kSize == sizeof(UncompiledDataWithoutPreparseDataWithJob));
};

// Definition https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=183&c=1
class TorqueGeneratedUncompiledDataWithPreparseDataAndJobAsserts {
  // https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=186&c=3
  static constexpr int kJobOffset = sizeof(UncompiledDataWithPreparseData);
  static constexpr int kJobOffsetEnd = kJobOffset + kSystemPointerSize - 1;
  static constexpr int kStartOfWeakFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kStartOfStrongFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kEndOfStrongFieldsOffset = kJobOffsetEnd + 1;
  static constexpr int kHeaderSize = kJobOffsetEnd + 1;
  static constexpr int kSize = kJobOffsetEnd + 1;

  static_assert(kJobOffset == offsetof(UncompiledDataWithPreparseDataAndJob, job_),
                "Value of UncompiledDataWithPreparseDataAndJob::kJobOffset defined in Torque and offset of field UncompiledDataWithPreparseDataAndJob::job in C++ do not match");
  static_assert(kSize == sizeof(UncompiledDataWithPreparseDataAndJob));
};

// https://crsrc.org/c/v8/src/objects/shared-function-info.tq?l=199&c=1
bool IsOnHeapBasicBlockProfilerData_NonInline(Tagged<HeapObject> o) {
  return IsOnHeapBasicBlockProfilerData(o);
}

#ifdef VERIFY_HEAP

template <>
void TorqueGeneratedOnHeapBasicBlockProfilerData<OnHeapBasicBlockProfilerData, HeapObject>::OnHeapBasicBlockProfilerDataVerify(Isolate* isolate) {
  TorqueGeneratedClassVerifiers::OnHeapBasicBlockProfilerDataVerify(TrustedCast<OnHeapBasicBlockProfilerData>(*this), isolate);
}


#endif  // VERIFY_HEAP
} // namespace internal
} // namespace v8
