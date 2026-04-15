class NewSloppyArgumentsElementsDescriptor : public StaticCallInterfaceDescriptor<NewSloppyArgumentsElementsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kFrame, kFormalParameterCount, kArgumentCount)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineTypeOf<RawPtrT>::value, MachineTypeOf<IntPtrT>::value, MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(NewSloppyArgumentsElementsDescriptor)
};

class NewStrictArgumentsElementsDescriptor : public StaticCallInterfaceDescriptor<NewStrictArgumentsElementsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kFrame, kFormalParameterCount, kArgumentCount)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineTypeOf<RawPtrT>::value, MachineTypeOf<IntPtrT>::value, MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(NewStrictArgumentsElementsDescriptor)
};

class NewRestArgumentsElementsDescriptor : public StaticCallInterfaceDescriptor<NewRestArgumentsElementsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kFrame, kFormalParameterCount, kArgumentCount)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineTypeOf<RawPtrT>::value, MachineTypeOf<IntPtrT>::value, MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(NewRestArgumentsElementsDescriptor)
};

class FastNewSloppyArgumentsDescriptor : public StaticCallInterfaceDescriptor<FastNewSloppyArgumentsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kF)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(FastNewSloppyArgumentsDescriptor)
};

class FastNewStrictArgumentsDescriptor : public StaticCallInterfaceDescriptor<FastNewStrictArgumentsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kF)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(FastNewStrictArgumentsDescriptor)
};

class FastNewRestArgumentsDescriptor : public StaticCallInterfaceDescriptor<FastNewRestArgumentsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kF)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(FastNewRestArgumentsDescriptor)
};

class StringFastLocaleCompareDescriptor : public StaticCallInterfaceDescriptor<StringFastLocaleCompareDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLocaleCompareFn, kLeft, kRight, kLocales)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StringFastLocaleCompareDescriptor)
};

class LoadFromContextCellDescriptor : public StaticCallInterfaceDescriptor<LoadFromContextCellDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kN, kC, kI)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(LoadFromContextCellDescriptor)
};

class StoreCurrentContextElementBaselineDescriptor : public StaticCallInterfaceDescriptor<StoreCurrentContextElementBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kO, kI)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(StoreCurrentContextElementBaselineDescriptor)
};

class StoreContextElementBaselineDescriptor : public StaticCallInterfaceDescriptor<StoreContextElementBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kC, kO, kI, kD)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreContextElementBaselineDescriptor)
};

class DetachContextCellDescriptor : public StaticCallInterfaceDescriptor<DetachContextCellDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kTheContext, kNewValue, kI)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineTypeOf<IntPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(DetachContextCellDescriptor)
};

class StringSlowFlattenDescriptor : public StaticCallInterfaceDescriptor<StringSlowFlattenDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kCons)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(StringSlowFlattenDescriptor)
};

class StringIndexOfDescriptor : public StaticCallInterfaceDescriptor<StringIndexOfDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kS, kSearchString, kStart)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(StringIndexOfDescriptor)
};

class CheckTurboshaftWord32TypeDescriptor : public StaticCallInterfaceDescriptor<CheckTurboshaftWord32TypeDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue, kExpectedType, kNodeId)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineTypeOf<Uint32T>::value, MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CheckTurboshaftWord32TypeDescriptor)
};

class CheckTurboshaftWord64TypeDescriptor : public StaticCallInterfaceDescriptor<CheckTurboshaftWord64TypeDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValueHigh, kValueLow, kExpectedType, kNodeId)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineTypeOf<Uint32T>::value, MachineTypeOf<Uint32T>::value, MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CheckTurboshaftWord64TypeDescriptor)
};

class DebugPrintObjectDescriptor : public StaticCallInterfaceDescriptor<DebugPrintObjectDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPrefix, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(DebugPrintObjectDescriptor)
};

class TestTurbofanTypeDescriptor : public StaticCallInterfaceDescriptor<TestTurbofanTypeDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue, kExpectedType)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(TestTurbofanTypeDescriptor)
};

class CheckTurbofanTypeDescriptor : public StaticCallInterfaceDescriptor<CheckTurbofanTypeDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue, kExpectedType, kNodeId)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CheckTurbofanTypeDescriptor)
};

class ProxyHasPropertyDescriptor : public StaticCallInterfaceDescriptor<ProxyHasPropertyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy, kName)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ProxyHasPropertyDescriptor)
};

class ArrayFindLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayFindLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kO, kInitialK, kLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayFindLoopContinuationDescriptor)
};

class ProxyDeletePropertyDescriptor : public StaticCallInterfaceDescriptor<ProxyDeletePropertyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy, kName, kLanguageMode)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(ProxyDeletePropertyDescriptor)
};

class PromiseResolveDescriptor : public StaticCallInterfaceDescriptor<PromiseResolveDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kConstructor, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(PromiseResolveDescriptor)
};

class ResolvePromiseDescriptor : public StaticCallInterfaceDescriptor<ResolvePromiseDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPromise, kResolution)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ResolvePromiseDescriptor)
};

class ArrayEveryLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayEveryLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kArray, kO, kInitialK, kLength, kInitialTo)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayEveryLoopContinuationDescriptor)
};

class GroupByGenericDescriptor : public StaticCallInterfaceDescriptor<GroupByGenericDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kItems, kInitialGroups, kCallbackfn, kCoerceToProperty, kMethodName)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(GroupByGenericDescriptor)
};

class ProxyIsExtensibleDescriptor : public StaticCallInterfaceDescriptor<ProxyIsExtensibleDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ProxyIsExtensibleDescriptor)
};

class ParseIntDescriptor : public StaticCallInterfaceDescriptor<ParseIntDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput, kRadix)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ParseIntDescriptor)
};

class AddDescriptor : public StaticCallInterfaceDescriptor<AddDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeftArg, kRightArg)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(AddDescriptor)
};

class SubtractDescriptor : public StaticCallInterfaceDescriptor<SubtractDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(SubtractDescriptor)
};

class MultiplyDescriptor : public StaticCallInterfaceDescriptor<MultiplyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(MultiplyDescriptor)
};

class DivideDescriptor : public StaticCallInterfaceDescriptor<DivideDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(DivideDescriptor)
};

class ModulusDescriptor : public StaticCallInterfaceDescriptor<ModulusDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ModulusDescriptor)
};

class ExponentiateDescriptor : public StaticCallInterfaceDescriptor<ExponentiateDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ExponentiateDescriptor)
};

class NegateDescriptor : public StaticCallInterfaceDescriptor<NegateDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(NegateDescriptor)
};

class BitwiseNotDescriptor : public StaticCallInterfaceDescriptor<BitwiseNotDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BitwiseNotDescriptor)
};

class DecrementDescriptor : public StaticCallInterfaceDescriptor<DecrementDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(DecrementDescriptor)
};

class IncrementDescriptor : public StaticCallInterfaceDescriptor<IncrementDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(IncrementDescriptor)
};

class ShiftLeftDescriptor : public StaticCallInterfaceDescriptor<ShiftLeftDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ShiftLeftDescriptor)
};

class ShiftRightDescriptor : public StaticCallInterfaceDescriptor<ShiftRightDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ShiftRightDescriptor)
};

class ShiftRightLogicalDescriptor : public StaticCallInterfaceDescriptor<ShiftRightLogicalDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ShiftRightLogicalDescriptor)
};

class BitwiseAndDescriptor : public StaticCallInterfaceDescriptor<BitwiseAndDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BitwiseAndDescriptor)
};

class BitwiseOrDescriptor : public StaticCallInterfaceDescriptor<BitwiseOrDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BitwiseOrDescriptor)
};

class BitwiseXorDescriptor : public StaticCallInterfaceDescriptor<BitwiseXorDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BitwiseXorDescriptor)
};

class LessThanDescriptor : public StaticCallInterfaceDescriptor<LessThanDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(LessThanDescriptor)
};

class LessThanOrEqualDescriptor : public StaticCallInterfaceDescriptor<LessThanOrEqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(LessThanOrEqualDescriptor)
};

class GreaterThanDescriptor : public StaticCallInterfaceDescriptor<GreaterThanDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GreaterThanDescriptor)
};

class GreaterThanOrEqualDescriptor : public StaticCallInterfaceDescriptor<GreaterThanOrEqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GreaterThanOrEqualDescriptor)
};

class EqualDescriptor : public StaticCallInterfaceDescriptor<EqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(EqualDescriptor)
};

class StrictEqualDescriptor : public StaticCallInterfaceDescriptor<StrictEqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StrictEqualDescriptor)
};

class PromiseResolveThenableJobDescriptor : public StaticCallInterfaceDescriptor<PromiseResolveThenableJobDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPromiseToResolve, kThenable, kThen)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(PromiseResolveThenableJobDescriptor)
};

class ArrayFilterLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayFilterLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kArray, kO, kInitialK, kLength, kInitialTo)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayFilterLoopContinuationDescriptor)
};

class StringToNumberDescriptor : public StaticCallInterfaceDescriptor<StringToNumberDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(StringToNumberDescriptor)
};

class NonNumberToNumberDescriptor : public StaticCallInterfaceDescriptor<NonNumberToNumberDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(NonNumberToNumberDescriptor)
};

class NonNumberToNumericDescriptor : public StaticCallInterfaceDescriptor<NonNumberToNumericDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(NonNumberToNumericDescriptor)
};

class ToNumericDescriptor : public StaticCallInterfaceDescriptor<ToNumericDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToNumericDescriptor)
};

class NumberToStringDescriptor : public StaticCallInterfaceDescriptor<NumberToStringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(NumberToStringDescriptor)
};

class SmiToStringDescriptor : public StaticCallInterfaceDescriptor<SmiToStringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(SmiToStringDescriptor)
};

class Int32ToStringDescriptor : public StaticCallInterfaceDescriptor<Int32ToStringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineTypeOf<Int32T>::value)
  DECLARE_DEFAULT_DESCRIPTOR(Int32ToStringDescriptor)
};

class ToBooleanDescriptor : public StaticCallInterfaceDescriptor<ToBooleanDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToBooleanDescriptor)
};

class ToBooleanForBaselineJumpDescriptor : public StaticCallInterfaceDescriptor<ToBooleanForBaselineJumpDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(2, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToBooleanForBaselineJumpDescriptor)
};

class ToLengthDescriptor : public StaticCallInterfaceDescriptor<ToLengthDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToLengthDescriptor)
};

class ToNameDescriptor : public StaticCallInterfaceDescriptor<ToNameDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToNameDescriptor)
};

class ToObjectDescriptor : public StaticCallInterfaceDescriptor<ToObjectDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToObjectDescriptor)
};

class NonPrimitiveToPrimitive_DefaultDescriptor : public StaticCallInterfaceDescriptor<NonPrimitiveToPrimitive_DefaultDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(NonPrimitiveToPrimitive_DefaultDescriptor)
};

class NonPrimitiveToPrimitive_NumberDescriptor : public StaticCallInterfaceDescriptor<NonPrimitiveToPrimitive_NumberDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(NonPrimitiveToPrimitive_NumberDescriptor)
};

class NonPrimitiveToPrimitive_StringDescriptor : public StaticCallInterfaceDescriptor<NonPrimitiveToPrimitive_StringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(NonPrimitiveToPrimitive_StringDescriptor)
};

class OrdinaryToPrimitive_NumberDescriptor : public StaticCallInterfaceDescriptor<OrdinaryToPrimitive_NumberDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(OrdinaryToPrimitive_NumberDescriptor)
};

class OrdinaryToPrimitive_Number_InlineDescriptor : public StaticCallInterfaceDescriptor<OrdinaryToPrimitive_Number_InlineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(OrdinaryToPrimitive_Number_InlineDescriptor)
};

class OrdinaryToPrimitive_StringDescriptor : public StaticCallInterfaceDescriptor<OrdinaryToPrimitive_StringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(OrdinaryToPrimitive_StringDescriptor)
};

class GenericArrayToReversedDescriptor : public StaticCallInterfaceDescriptor<GenericArrayToReversedDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GenericArrayToReversedDescriptor)
};

class ProxySetPropertyDescriptor : public StaticCallInterfaceDescriptor<ProxySetPropertyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy, kName, kValue, kReceiverValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ProxySetPropertyDescriptor)
};

class RegExpReplaceDescriptor : public StaticCallInterfaceDescriptor<RegExpReplaceDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kRegexp, kString, kReplaceValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(RegExpReplaceDescriptor)
};

class StringRepeatDescriptor : public StaticCallInterfaceDescriptor<StringRepeatDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kString, kCount)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(StringRepeatDescriptor)
};

class FastNewClosureBaselineDescriptor : public StaticCallInterfaceDescriptor<FastNewClosureBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kSharedFunctionInfo, kFeedbackCell)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(FastNewClosureBaselineDescriptor)
};

class FastNewFunctionContextEvalDescriptor : public StaticCallInterfaceDescriptor<FastNewFunctionContextEvalDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kScopeInfo, kSlots)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineTypeOf<Uint32T>::value)
  DECLARE_DEFAULT_DESCRIPTOR(FastNewFunctionContextEvalDescriptor)
};

class FastNewFunctionContextFunctionDescriptor : public StaticCallInterfaceDescriptor<FastNewFunctionContextFunctionDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kScopeInfo, kSlots)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineTypeOf<Uint32T>::value)
  DECLARE_DEFAULT_DESCRIPTOR(FastNewFunctionContextFunctionDescriptor)
};

class FastNewFunctionContextFunctionWithCellsDescriptor : public StaticCallInterfaceDescriptor<FastNewFunctionContextFunctionWithCellsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kScopeInfo, kSlots)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineTypeOf<Uint32T>::value)
  DECLARE_DEFAULT_DESCRIPTOR(FastNewFunctionContextFunctionWithCellsDescriptor)
};

class CreateRegExpLiteralDescriptor : public StaticCallInterfaceDescriptor<CreateRegExpLiteralDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kMaybeFeedbackVector, kSlot, kPattern, kFlags)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CreateRegExpLiteralDescriptor)
};

class CreateShallowArrayLiteralDescriptor : public StaticCallInterfaceDescriptor<CreateShallowArrayLiteralDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kMaybeFeedbackVector, kSlot, kConstantElements, kFlags)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CreateShallowArrayLiteralDescriptor)
};

class CreateEmptyArrayLiteralDescriptor : public StaticCallInterfaceDescriptor<CreateEmptyArrayLiteralDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kFeedbackVector, kSlot)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CreateEmptyArrayLiteralDescriptor)
};

class CreateShallowObjectLiteralDescriptor : public StaticCallInterfaceDescriptor<CreateShallowObjectLiteralDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kMaybeFeedbackVector, kSlot, kDesc, kFlags)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CreateShallowObjectLiteralDescriptor)
};

class CreateEmptyLiteralObjectDescriptor : public StaticCallInterfaceDescriptor<CreateEmptyLiteralObjectDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CreateEmptyLiteralObjectDescriptor)
};

class BigIntAddNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntAddNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntAddNoThrowDescriptor)
};

class BigIntAddDescriptor : public StaticCallInterfaceDescriptor<BigIntAddDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntAddDescriptor)
};

class BigIntSubtractNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntSubtractNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntSubtractNoThrowDescriptor)
};

class BigIntSubtractDescriptor : public StaticCallInterfaceDescriptor<BigIntSubtractDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntSubtractDescriptor)
};

class BigIntMultiplyNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntMultiplyNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntMultiplyNoThrowDescriptor)
};

class BigIntMultiplyDescriptor : public StaticCallInterfaceDescriptor<BigIntMultiplyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntMultiplyDescriptor)
};

class BigIntDivideNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntDivideNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntDivideNoThrowDescriptor)
};

class BigIntDivideDescriptor : public StaticCallInterfaceDescriptor<BigIntDivideDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntDivideDescriptor)
};

class BigIntModulusNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntModulusNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntModulusNoThrowDescriptor)
};

class BigIntModulusDescriptor : public StaticCallInterfaceDescriptor<BigIntModulusDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntModulusDescriptor)
};

class BigIntBitwiseAndNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntBitwiseAndNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntBitwiseAndNoThrowDescriptor)
};

class BigIntBitwiseAndDescriptor : public StaticCallInterfaceDescriptor<BigIntBitwiseAndDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntBitwiseAndDescriptor)
};

class BigIntBitwiseOrNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntBitwiseOrNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntBitwiseOrNoThrowDescriptor)
};

class BigIntBitwiseOrDescriptor : public StaticCallInterfaceDescriptor<BigIntBitwiseOrDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntBitwiseOrDescriptor)
};

class BigIntBitwiseXorNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntBitwiseXorNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntBitwiseXorNoThrowDescriptor)
};

class BigIntBitwiseXorDescriptor : public StaticCallInterfaceDescriptor<BigIntBitwiseXorDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntBitwiseXorDescriptor)
};

class BigIntShiftLeftNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntShiftLeftNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntShiftLeftNoThrowDescriptor)
};

class BigIntShiftLeftDescriptor : public StaticCallInterfaceDescriptor<BigIntShiftLeftDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntShiftLeftDescriptor)
};

class BigIntShiftRightNoThrowDescriptor : public StaticCallInterfaceDescriptor<BigIntShiftRightNoThrowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntShiftRightNoThrowDescriptor)
};

class BigIntShiftRightDescriptor : public StaticCallInterfaceDescriptor<BigIntShiftRightDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kXNum, kYNum)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntShiftRightDescriptor)
};

class BigIntEqualDescriptor : public StaticCallInterfaceDescriptor<BigIntEqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntEqualDescriptor)
};

class BigIntLessThanDescriptor : public StaticCallInterfaceDescriptor<BigIntLessThanDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntLessThanDescriptor)
};

class BigIntGreaterThanDescriptor : public StaticCallInterfaceDescriptor<BigIntGreaterThanDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntGreaterThanDescriptor)
};

class BigIntLessThanOrEqualDescriptor : public StaticCallInterfaceDescriptor<BigIntLessThanOrEqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntLessThanOrEqualDescriptor)
};

class BigIntGreaterThanOrEqualDescriptor : public StaticCallInterfaceDescriptor<BigIntGreaterThanOrEqualDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntGreaterThanOrEqualDescriptor)
};

class BigIntUnaryMinusDescriptor : public StaticCallInterfaceDescriptor<BigIntUnaryMinusDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kBigint)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(BigIntUnaryMinusDescriptor)
};

class CreateObjectWithoutPropertiesDescriptor : public StaticCallInterfaceDescriptor<CreateObjectWithoutPropertiesDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPrototype)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CreateObjectWithoutPropertiesDescriptor)
};

class ProxyGetPrototypeOfDescriptor : public StaticCallInterfaceDescriptor<ProxyGetPrototypeOfDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ProxyGetPrototypeOfDescriptor)
};

class ArrayReduceRightLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayReduceRightLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kInitialAccumulator, kO, kInitialK, kLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayReduceRightLoopContinuationDescriptor)
};

class ArrayFindIndexLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayFindIndexLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kO, kInitialK, kLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayFindIndexLoopContinuationDescriptor)
};

class ArrayForEachLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayForEachLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kArray, kO, kInitialK, kLen, kTo)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayForEachLoopContinuationDescriptor)
};

class CreateHTMLDescriptor : public StaticCallInterfaceDescriptor<CreateHTMLDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kMethodName, kTagName, kAttr, kAttrValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CreateHTMLDescriptor)
};

class ArrayFindLastIndexLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayFindLastIndexLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPredicate, kThisArg, kO, kInitialK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayFindLastIndexLoopContinuationDescriptor)
};

class GetTemplateObjectDescriptor : public StaticCallInterfaceDescriptor<GetTemplateObjectDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kShared, kDescription, kSlot, kMaybeFeedbackVector)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(GetTemplateObjectDescriptor)
};

class ForInNextDescriptor : public StaticCallInterfaceDescriptor<ForInNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSlot, kReceiver, kCacheArray, kCacheType, kCacheIndex, kFeedbackVector)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineTypeOf<UintPtrT>::value, MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ForInNextDescriptor)
};

class GetImportMetaObjectBaselineDescriptor : public StaticCallInterfaceDescriptor<GetImportMetaObjectBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GetImportMetaObjectBaselineDescriptor)
};

class IteratorMapHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorMapHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorMapHelperNextDescriptor)
};

class IteratorFilterHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorFilterHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorFilterHelperNextDescriptor)
};

class IteratorTakeHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorTakeHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorTakeHelperNextDescriptor)
};

class IteratorDropHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorDropHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorDropHelperNextDescriptor)
};

class IteratorFlatMapHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorFlatMapHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorFlatMapHelperNextDescriptor)
};

class IteratorConcatHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorConcatHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorConcatHelperNextDescriptor)
};

class IteratorZipHelperNextDescriptor : public StaticCallInterfaceDescriptor<IteratorZipHelperNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kHelper)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(IteratorZipHelperNextDescriptor)
};

class PromiseFulfillReactionJobDescriptor : public StaticCallInterfaceDescriptor<PromiseFulfillReactionJobDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue, kHandler, kPromiseOrCapability)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(PromiseFulfillReactionJobDescriptor)
};

class PromiseRejectReactionJobDescriptor : public StaticCallInterfaceDescriptor<PromiseRejectReactionJobDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReason, kHandler, kPromiseOrCapability)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(PromiseRejectReactionJobDescriptor)
};

class TypedArrayMergeSortDescriptor : public StaticCallInterfaceDescriptor<TypedArrayMergeSortDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSource, kFrom, kTo, kTarget, kArray, kComparefn)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineTypeOf<UintPtrT>::value, MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(TypedArrayMergeSortDescriptor)
};

class ArrayTimSortIntoCopyDescriptor : public StaticCallInterfaceDescriptor<ArrayTimSortIntoCopyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayTimSortIntoCopyDescriptor)
};

class ToIntegerDescriptor : public StaticCallInterfaceDescriptor<ToIntegerDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kInput)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToIntegerDescriptor)
};

class FastCreateDataPropertyDescriptor : public StaticCallInterfaceDescriptor<FastCreateDataPropertyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kKey, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(FastCreateDataPropertyDescriptor)
};

class CreateTypedArrayDescriptor : public StaticCallInterfaceDescriptor<CreateTypedArrayDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTarget, kNewTarget, kArg1, kArg2, kArg3)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CreateTypedArrayDescriptor)
};

class ArraySomeLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArraySomeLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kArray, kO, kInitialK, kLength, kInitialTo)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArraySomeLoopContinuationDescriptor)
};

class ArrayFindLastLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayFindLastLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPredicate, kThisArg, kO, kInitialK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayFindLastLoopContinuationDescriptor)
};

class ProxyGetPropertyDescriptor : public StaticCallInterfaceDescriptor<ProxyGetPropertyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy, kName, kReceiverValue, kOnNonExistent)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(ProxyGetPropertyDescriptor)
};

class GenericArrayWithDescriptor : public StaticCallInterfaceDescriptor<GenericArrayWithDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kLen, kActualIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GenericArrayWithDescriptor)
};

class AsyncAwaitNonThenableFastPathDescriptor : public StaticCallInterfaceDescriptor<AsyncAwaitNonThenableFastPathDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue, kOnResolve, kThrowaway)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(AsyncAwaitNonThenableFastPathDescriptor)
};

class ProxyPreventExtensionsDescriptor : public StaticCallInterfaceDescriptor<ProxyPreventExtensionsDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy, kDoThrow)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ProxyPreventExtensionsDescriptor)
};

class IncBlockCounterDescriptor : public StaticCallInterfaceDescriptor<IncBlockCounterDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kFunction, kCoverageArraySlotIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(IncBlockCounterDescriptor)
};

class RegExpSearchFastDescriptor : public StaticCallInterfaceDescriptor<RegExpSearchFastDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kString)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(RegExpSearchFastDescriptor)
};

class ArrayReduceLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayReduceLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kInitialAccumulator, kO, kInitialK, kLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayReduceLoopContinuationDescriptor)
};

class RegExpSplitDescriptor : public StaticCallInterfaceDescriptor<RegExpSplitDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kRegexp, kString, kLimit)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(RegExpSplitDescriptor)
};

class LoadJoinElement_DictionaryElements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinElement_DictionaryElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinElement_DictionaryElements_0Descriptor)
};

class LoadJoinElement_FastSmiOrObjectElements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinElement_FastSmiOrObjectElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinElement_FastSmiOrObjectElements_0Descriptor)
};

class LoadJoinElement_FastDoubleElements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinElement_FastDoubleElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinElement_FastDoubleElements_0Descriptor)
};

class ConvertToLocaleStringDescriptor : public StaticCallInterfaceDescriptor<ConvertToLocaleStringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kElement, kLocales, kOptions)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ConvertToLocaleStringDescriptor)
};

class JoinStackPushDescriptor : public StaticCallInterfaceDescriptor<JoinStackPushDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kStack, kReceiver)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(JoinStackPushDescriptor)
};

class JoinStackPopDescriptor : public StaticCallInterfaceDescriptor<JoinStackPopDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kStack, kReceiver)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(JoinStackPopDescriptor)
};

class ArrayPrototypeJoinImplDescriptor : public StaticCallInterfaceDescriptor<ArrayPrototypeJoinImplDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kO, kLen, kSeparator)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayPrototypeJoinImplDescriptor)
};

class ArrayMapLoopContinuationDescriptor : public StaticCallInterfaceDescriptor<ArrayMapLoopContinuationDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kCallbackfn, kThisArg, kArray, kO, kInitialK, kLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayMapLoopContinuationDescriptor)
};

class RegExpPrototypeExecSlowDescriptor : public StaticCallInterfaceDescriptor<RegExpPrototypeExecSlowDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kRegexp, kString)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(RegExpPrototypeExecSlowDescriptor)
};

class FulfillPromiseDescriptor : public StaticCallInterfaceDescriptor<FulfillPromiseDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPromise, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(FulfillPromiseDescriptor)
};

class RejectPromiseDescriptor : public StaticCallInterfaceDescriptor<RejectPromiseDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPromise, kReason, kDebugEvent)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(RejectPromiseDescriptor)
};

class NewPromiseCapabilityDescriptor : public StaticCallInterfaceDescriptor<NewPromiseCapabilityDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kMaybeConstructor, kDebugEvent)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(NewPromiseCapabilityDescriptor)
};

class PerformPromiseThenDescriptor : public StaticCallInterfaceDescriptor<PerformPromiseThenDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kPromise, kOnFulfilled, kOnRejected, kResultPromise)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(PerformPromiseThenDescriptor)
};

class FlattenIntoArrayWithoutMapFnDescriptor : public StaticCallInterfaceDescriptor<FlattenIntoArrayWithoutMapFnDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTarget, kSource, kSourceLength, kStart, kDepth)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(FlattenIntoArrayWithoutMapFnDescriptor)
};

class FlattenIntoArrayWithMapFnDescriptor : public StaticCallInterfaceDescriptor<FlattenIntoArrayWithMapFnDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTarget, kSource, kSourceLength, kStart, kDepth, kMapfn, kThisArgs)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(FlattenIntoArrayWithMapFnDescriptor)
};

class RegExpMatchFastDescriptor : public StaticCallInterfaceDescriptor<RegExpMatchFastDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kString)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(RegExpMatchFastDescriptor)
};

class RegExpPrototypeTestFastDescriptor : public StaticCallInterfaceDescriptor<RegExpPrototypeTestFastDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kString)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(RegExpPrototypeTestFastDescriptor)
};

class ToStringDescriptor : public StaticCallInterfaceDescriptor<ToStringDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kO)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToStringDescriptor)
};

class ToStringConvertSymbolDescriptor : public StaticCallInterfaceDescriptor<ToStringConvertSymbolDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kO)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(ToStringConvertSymbolDescriptor)
};

class StringToListDescriptor : public StaticCallInterfaceDescriptor<StringToListDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kString)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(StringToListDescriptor)
};

class StringAddConvertLeftDescriptor : public StaticCallInterfaceDescriptor<StringAddConvertLeftDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(StringAddConvertLeftDescriptor)
};

class StringAddConvertRightDescriptor : public StaticCallInterfaceDescriptor<StringAddConvertRightDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kLeft, kRight)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StringAddConvertRightDescriptor)
};

class StringCharAtDescriptor : public StaticCallInterfaceDescriptor<StringCharAtDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kPosition)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(StringCharAtDescriptor)
};

class ProxySetPrototypeOfDescriptor : public StaticCallInterfaceDescriptor<ProxySetPrototypeOfDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kProxy, kProto, kDoThrow)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ProxySetPrototypeOfDescriptor)
};

class GetIteratorWithFeedbackDescriptor : public StaticCallInterfaceDescriptor<GetIteratorWithFeedbackDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kLoadSlot, kCallSlot, kMaybeFeedbackVector)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(GetIteratorWithFeedbackDescriptor)
};

class GetIteratorBaselineDescriptor : public StaticCallInterfaceDescriptor<GetIteratorBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kReceiver, kLoadSlot, kCallSlot)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GetIteratorBaselineDescriptor)
};

class CreateAsyncFromSyncIteratorBaselineDescriptor : public StaticCallInterfaceDescriptor<CreateAsyncFromSyncIteratorBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kSyncIterator)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CreateAsyncFromSyncIteratorBaselineDescriptor)
};

class ForOfNextDescriptor : public StaticCallInterfaceDescriptor<ForOfNextDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(2, kIterator, kNext, kFeedbackVector, kCallSlot)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(ForOfNextDescriptor)
};

class ForOfNextBaselineDescriptor : public StaticCallInterfaceDescriptor<ForOfNextBaselineDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kIterator, kNext, kResultSlot, kCallSlot)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<RawPtrT>::value, MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(ForOfNextBaselineDescriptor)
};

class CallIteratorWithFeedbackDescriptor : public StaticCallInterfaceDescriptor<CallIteratorWithFeedbackDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kIteratorMethod, kCallSlot, kFeedback)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(CallIteratorWithFeedbackDescriptor)
};

class Load_FastSmiElements_0Descriptor : public StaticCallInterfaceDescriptor<Load_FastSmiElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Load_FastSmiElements_0Descriptor)
};

class Load_FastObjectElements_0Descriptor : public StaticCallInterfaceDescriptor<Load_FastObjectElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Load_FastObjectElements_0Descriptor)
};

class Load_FastDoubleElements_0Descriptor : public StaticCallInterfaceDescriptor<Load_FastDoubleElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Load_FastDoubleElements_0Descriptor)
};

class Store_FastSmiElements_0Descriptor : public StaticCallInterfaceDescriptor<Store_FastSmiElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(Store_FastSmiElements_0Descriptor)
};

class Store_FastObjectElements_0Descriptor : public StaticCallInterfaceDescriptor<Store_FastObjectElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(Store_FastObjectElements_0Descriptor)
};

class Store_FastDoubleElements_0Descriptor : public StaticCallInterfaceDescriptor<Store_FastDoubleElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(Store_FastDoubleElements_0Descriptor)
};

class Delete_FastSmiElements_0Descriptor : public StaticCallInterfaceDescriptor<Delete_FastSmiElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Delete_FastSmiElements_0Descriptor)
};

class Delete_FastObjectElements_0Descriptor : public StaticCallInterfaceDescriptor<Delete_FastObjectElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Delete_FastObjectElements_0Descriptor)
};

class Delete_FastDoubleElements_0Descriptor : public StaticCallInterfaceDescriptor<Delete_FastDoubleElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Delete_FastDoubleElements_0Descriptor)
};

class SortCompareDefaultDescriptor : public StaticCallInterfaceDescriptor<SortCompareDefaultDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kComparefn, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(SortCompareDefaultDescriptor)
};

class SortCompareUserFnDescriptor : public StaticCallInterfaceDescriptor<SortCompareUserFnDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kComparefn, kX, kY)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(SortCompareUserFnDescriptor)
};

class CanUseSameAccessor_GenericElementsAccessor_0Descriptor : public StaticCallInterfaceDescriptor<CanUseSameAccessor_GenericElementsAccessor_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kInitialReceiverMap, kInitialReceiverLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CanUseSameAccessor_GenericElementsAccessor_0Descriptor)
};

class CopyDescriptor : public StaticCallInterfaceDescriptor<CopyDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSource, kSrcPos, kTarget, kDstPos, kLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(CopyDescriptor)
};

class MergeAtDescriptor : public StaticCallInterfaceDescriptor<MergeAtDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kI)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(MergeAtDescriptor)
};

class GallopLeftDescriptor : public StaticCallInterfaceDescriptor<GallopLeftDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kArray, kKey, kBase, kLength, kHint)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::TaggedSigned(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(GallopLeftDescriptor)
};

class GallopRightDescriptor : public StaticCallInterfaceDescriptor<GallopRightDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kArray, kKey, kBase, kLength, kHint)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged(), MachineType::TaggedSigned(), MachineType::TaggedSigned(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(GallopRightDescriptor)
};

class ArrayTimSortDescriptor : public StaticCallInterfaceDescriptor<ArrayTimSortDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ArrayTimSortDescriptor)
};

class GenericBuiltinTest_JSAny_0Descriptor : public StaticCallInterfaceDescriptor<GenericBuiltinTest_JSAny_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kParam)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(GenericBuiltinTest_JSAny_0Descriptor)
};

class TestHelperPlus1Descriptor : public StaticCallInterfaceDescriptor<TestHelperPlus1Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kX)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(TestHelperPlus1Descriptor)
};

class TestHelperPlus2Descriptor : public StaticCallInterfaceDescriptor<TestHelperPlus2Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kX)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(TestHelperPlus2Descriptor)
};

class NewSmiBoxDescriptor : public StaticCallInterfaceDescriptor<NewSmiBoxDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(NewSmiBoxDescriptor)
};

class ReturnTwoValuesDescriptor : public StaticCallInterfaceDescriptor<ReturnTwoValuesDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(2, kValue, kObj)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ReturnTwoValuesDescriptor)
};

class ThrowAsBuiltinDescriptor : public StaticCallInterfaceDescriptor<ThrowAsBuiltinDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(0, kStr)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer())
  DECLARE_DEFAULT_DESCRIPTOR(ThrowAsBuiltinDescriptor)
};

class TestCallNeverDescriptor : public StaticCallInterfaceDescriptor<TestCallNeverDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kA)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(TestCallNeverDescriptor)
};

class TestIncrementArraySpeciesModifiedDescriptor : public StaticCallInterfaceDescriptor<TestIncrementArraySpeciesModifiedDescriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(TestIncrementArraySpeciesModifiedDescriptor)
};

class LoadJoinElement_GenericElementsAccessor_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinElement_GenericElementsAccessor_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinElement_GenericElementsAccessor_0Descriptor)
};

class LoadJoinTypedElement_Int32Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Int32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Int32Elements_0Descriptor)
};

class LoadJoinTypedElement_Float16Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Float16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Float16Elements_0Descriptor)
};

class LoadJoinTypedElement_Float32Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Float32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Float32Elements_0Descriptor)
};

class LoadJoinTypedElement_Float64Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Float64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Float64Elements_0Descriptor)
};

class LoadJoinTypedElement_Uint8ClampedElements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Uint8ClampedElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Uint8ClampedElements_0Descriptor)
};

class LoadJoinTypedElement_BigUint64Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_BigUint64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_BigUint64Elements_0Descriptor)
};

class LoadJoinTypedElement_BigInt64Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_BigInt64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_BigInt64Elements_0Descriptor)
};

class LoadJoinTypedElement_Uint8Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Uint8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Uint8Elements_0Descriptor)
};

class LoadJoinTypedElement_Int8Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Int8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Int8Elements_0Descriptor)
};

class LoadJoinTypedElement_Uint16Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Uint16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Uint16Elements_0Descriptor)
};

class LoadJoinTypedElement_Int16Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Int16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Int16Elements_0Descriptor)
};

class LoadJoinTypedElement_Uint32Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadJoinTypedElement_Uint32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kK)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadJoinTypedElement_Uint32Elements_0Descriptor)
};

class LoadNoHasPropertyCheck_GenericElementsAccessor_0Descriptor : public StaticCallInterfaceDescriptor<LoadNoHasPropertyCheck_GenericElementsAccessor_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(LoadNoHasPropertyCheck_GenericElementsAccessor_0Descriptor)
};

class Load_GenericElementsAccessor_0Descriptor : public StaticCallInterfaceDescriptor<Load_GenericElementsAccessor_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Load_GenericElementsAccessor_0Descriptor)
};

class Store_GenericElementsAccessor_0Descriptor : public StaticCallInterfaceDescriptor<Store_GenericElementsAccessor_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(Store_GenericElementsAccessor_0Descriptor)
};

class Delete_GenericElementsAccessor_0Descriptor : public StaticCallInterfaceDescriptor<Delete_GenericElementsAccessor_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kSortState, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(Delete_GenericElementsAccessor_0Descriptor)
};

class GenericBuiltinTest_Smi_0Descriptor : public StaticCallInterfaceDescriptor<GenericBuiltinTest_Smi_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kParam)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedSigned())
  DECLARE_DEFAULT_DESCRIPTOR(GenericBuiltinTest_Smi_0Descriptor)
};

class CanUseSameAccessor_FastDoubleElements_0Descriptor : public StaticCallInterfaceDescriptor<CanUseSameAccessor_FastDoubleElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kInitialReceiverMap, kInitialReceiverLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CanUseSameAccessor_FastDoubleElements_0Descriptor)
};

class CanUseSameAccessor_FastSmiElements_0Descriptor : public StaticCallInterfaceDescriptor<CanUseSameAccessor_FastSmiElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kInitialReceiverMap, kInitialReceiverLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CanUseSameAccessor_FastSmiElements_0Descriptor)
};

class CanUseSameAccessor_FastObjectElements_0Descriptor : public StaticCallInterfaceDescriptor<CanUseSameAccessor_FastObjectElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kReceiver, kInitialReceiverMap, kInitialReceiverLength)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::TaggedPointer(), MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(CanUseSameAccessor_FastObjectElements_0Descriptor)
};

class LoadTypedElement_Int32Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Int32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Int32Elements_0Descriptor)
};

class StoreTypedElementNumeric_Int32Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Int32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Int32Elements_0Descriptor)
};

class StoreTypedElementJSAny_Int32Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Int32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Int32Elements_0Descriptor)
};

class LoadTypedElement_Float16Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Float16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Float16Elements_0Descriptor)
};

class StoreTypedElementNumeric_Float16Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Float16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Float16Elements_0Descriptor)
};

class StoreTypedElementJSAny_Float16Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Float16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Float16Elements_0Descriptor)
};

class LoadTypedElement_Float32Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Float32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Float32Elements_0Descriptor)
};

class StoreTypedElementNumeric_Float32Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Float32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Float32Elements_0Descriptor)
};

class StoreTypedElementJSAny_Float32Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Float32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Float32Elements_0Descriptor)
};

class LoadTypedElement_Float64Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Float64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Float64Elements_0Descriptor)
};

class StoreTypedElementNumeric_Float64Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Float64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Float64Elements_0Descriptor)
};

class StoreTypedElementJSAny_Float64Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Float64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Float64Elements_0Descriptor)
};

class LoadTypedElement_Uint8ClampedElements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Uint8ClampedElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Uint8ClampedElements_0Descriptor)
};

class StoreTypedElementNumeric_Uint8ClampedElements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Uint8ClampedElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Uint8ClampedElements_0Descriptor)
};

class StoreTypedElementJSAny_Uint8ClampedElements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Uint8ClampedElements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Uint8ClampedElements_0Descriptor)
};

class LoadTypedElement_BigUint64Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_BigUint64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_BigUint64Elements_0Descriptor)
};

class StoreTypedElementNumeric_BigUint64Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_BigUint64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_BigUint64Elements_0Descriptor)
};

class StoreTypedElementJSAny_BigUint64Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_BigUint64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_BigUint64Elements_0Descriptor)
};

class LoadTypedElement_BigInt64Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_BigInt64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_BigInt64Elements_0Descriptor)
};

class StoreTypedElementNumeric_BigInt64Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_BigInt64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_BigInt64Elements_0Descriptor)
};

class StoreTypedElementJSAny_BigInt64Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_BigInt64Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_BigInt64Elements_0Descriptor)
};

class LoadTypedElement_Uint8Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Uint8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Uint8Elements_0Descriptor)
};

class StoreTypedElementNumeric_Uint8Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Uint8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Uint8Elements_0Descriptor)
};

class StoreTypedElementJSAny_Uint8Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Uint8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Uint8Elements_0Descriptor)
};

class LoadTypedElement_Int8Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Int8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Int8Elements_0Descriptor)
};

class StoreTypedElementNumeric_Int8Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Int8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Int8Elements_0Descriptor)
};

class StoreTypedElementJSAny_Int8Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Int8Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Int8Elements_0Descriptor)
};

class LoadTypedElement_Uint16Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Uint16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Uint16Elements_0Descriptor)
};

class StoreTypedElementNumeric_Uint16Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Uint16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Uint16Elements_0Descriptor)
};

class StoreTypedElementJSAny_Uint16Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Uint16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Uint16Elements_0Descriptor)
};

class LoadTypedElement_Int16Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Int16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Int16Elements_0Descriptor)
};

class StoreTypedElementNumeric_Int16Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Int16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Int16Elements_0Descriptor)
};

class StoreTypedElementJSAny_Int16Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Int16Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Int16Elements_0Descriptor)
};

class LoadTypedElement_Uint32Elements_0Descriptor : public StaticCallInterfaceDescriptor<LoadTypedElement_Uint32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS_NO_CONTEXT(1, kArray, kIndex)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::AnyTagged(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value)
  DECLARE_DEFAULT_DESCRIPTOR(LoadTypedElement_Uint32Elements_0Descriptor)
};

class StoreTypedElementNumeric_Uint32Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementNumeric_Uint32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementNumeric_Uint32Elements_0Descriptor)
};

class StoreTypedElementJSAny_Uint32Elements_0Descriptor : public StaticCallInterfaceDescriptor<StoreTypedElementJSAny_Uint32Elements_0Descriptor> {
 public:
  INTERNAL_DESCRIPTOR()
  SANDBOXING_MODE(kSandboxed)
  DEFINE_RESULT_AND_PARAMETERS(1, kTypedArray, kIndex, kValue)
  DEFINE_RESULT_AND_PARAMETER_TYPES(MachineType::TaggedSigned(), MachineType::TaggedPointer(), MachineTypeOf<UintPtrT>::value, MachineType::AnyTagged())
  DECLARE_DEFAULT_DESCRIPTOR(StoreTypedElementJSAny_Uint32Elements_0Descriptor)
};

