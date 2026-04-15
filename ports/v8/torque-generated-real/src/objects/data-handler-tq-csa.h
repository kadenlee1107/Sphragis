#ifndef V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_DATA_HANDLER_TQ_CSA_H_
#define V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_DATA_HANDLER_TQ_CSA_H_

#include "src/builtins/torque-csa-header-includes.h"

namespace v8 {
namespace internal {

// https://crsrc.org/c/v8/src/objects/data-handler.tq?l=5&c=1
TNode<DataHandler> Cast_DataHandler_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_obj, compiler::CodeAssemblerLabel* label_CastError);

// https://crsrc.org/c/v8/src/objects/data-handler.tq?l=8&c=3
TNode<Union<Code, Smi>> LoadDataHandlerSmiHandler_0(compiler::CodeAssemblerState* state_, TNode<DataHandler> p_o);

// https://crsrc.org/c/v8/src/objects/data-handler.tq?l=8&c=3
void StoreDataHandlerSmiHandler_0(compiler::CodeAssemblerState* state_, TNode<DataHandler> p_o, TNode<Union<Code, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/data-handler.tq?l=9&c=3
TNode<Union<Cell, Smi>> LoadDataHandlerValidityCell_0(compiler::CodeAssemblerState* state_, TNode<DataHandler> p_o);

// https://crsrc.org/c/v8/src/objects/data-handler.tq?l=9&c=3
void StoreDataHandlerValidityCell_0(compiler::CodeAssemblerState* state_, TNode<DataHandler> p_o, TNode<Union<Cell, Smi>> p_v);

// https://crsrc.org/c/v8/src/objects/data-handler.tq?l=5&c=1
TNode<DataHandler> DownCastForTorqueClass_DataHandler_0(compiler::CodeAssemblerState* state_, TNode<HeapObject> p_o, compiler::CodeAssemblerLabel* label_CastError);

} // namespace internal
} // namespace v8

#endif // V8_GEN_TORQUE_GENERATED_SRC_OBJECTS_DATA_HANDLER_TQ_CSA_H_
