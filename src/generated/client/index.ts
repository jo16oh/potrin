import { z } from 'zod';
import type { Prisma } from './prismaClient';
import { type TableSchema, DbSchema, ElectricClient, type HKT } from 'electric-sql/client/model';
import migrations from './migrations';
import pgMigrations from './pg-migrations';

/////////////////////////////////////////
// HELPER FUNCTIONS
/////////////////////////////////////////


/////////////////////////////////////////
// ENUMS
/////////////////////////////////////////

export const TransactionIsolationLevelSchema = z.enum(['ReadUncommitted','ReadCommitted','RepeatableRead','Serializable']);

export const CardsScalarFieldEnumSchema = z.enum(['id','thread','prev_card','content','created_at','updated_at']);

export const ThreadsScalarFieldEnumSchema = z.enum(['id','parent_thread','prev_thread','title','created_at','updated_at','deleted']);

export const SortOrderSchema = z.enum(['asc','desc']);

export const QueryModeSchema = z.enum(['default','insensitive']);

export const NullsOrderSchema = z.enum(['first','last']);
/////////////////////////////////////////
// MODELS
/////////////////////////////////////////

/////////////////////////////////////////
// CARDS SCHEMA
/////////////////////////////////////////

export const CardsSchema = z.object({
  id: z.string().uuid(),
  thread: z.string().uuid().nullable(),
  prev_card: z.string().uuid().nullable(),
  content: z.string().nullable(),
  created_at: z.coerce.date().nullable(),
  updated_at: z.coerce.date().nullable(),
})

export type Cards = z.infer<typeof CardsSchema>

/////////////////////////////////////////
// THREADS SCHEMA
/////////////////////////////////////////

export const ThreadsSchema = z.object({
  id: z.string().uuid(),
  parent_thread: z.string().uuid().nullable(),
  prev_thread: z.string().uuid().nullable(),
  title: z.string().nullable(),
  created_at: z.coerce.date().nullable(),
  updated_at: z.coerce.date().nullable(),
  deleted: z.boolean().nullable(),
})

export type Threads = z.infer<typeof ThreadsSchema>

/////////////////////////////////////////
// SELECT & INCLUDE
/////////////////////////////////////////

// CARDS
//------------------------------------------------------

export const CardsSelectSchema: z.ZodType<Prisma.CardsSelect> = z.object({
  id: z.boolean().optional(),
  thread: z.boolean().optional(),
  prev_card: z.boolean().optional(),
  content: z.boolean().optional(),
  created_at: z.boolean().optional(),
  updated_at: z.boolean().optional(),
}).strict()

// THREADS
//------------------------------------------------------

export const ThreadsSelectSchema: z.ZodType<Prisma.ThreadsSelect> = z.object({
  id: z.boolean().optional(),
  parent_thread: z.boolean().optional(),
  prev_thread: z.boolean().optional(),
  title: z.boolean().optional(),
  created_at: z.boolean().optional(),
  updated_at: z.boolean().optional(),
  deleted: z.boolean().optional(),
}).strict()

// CREATE MANY CARDS AND RETURN OUTPUT TYPE
//------------------------------------------------------

export const CreateManyCardsAndReturnOutputTypeSelectSchema: z.ZodType<Prisma.CreateManyCardsAndReturnOutputTypeSelect> = z.object({
  id: z.boolean().optional(),
  thread: z.boolean().optional(),
  prev_card: z.boolean().optional(),
  content: z.boolean().optional(),
  created_at: z.boolean().optional(),
  updated_at: z.boolean().optional(),
}).strict()

// CREATE MANY THREADS AND RETURN OUTPUT TYPE
//------------------------------------------------------

export const CreateManyThreadsAndReturnOutputTypeSelectSchema: z.ZodType<Prisma.CreateManyThreadsAndReturnOutputTypeSelect> = z.object({
  id: z.boolean().optional(),
  parent_thread: z.boolean().optional(),
  prev_thread: z.boolean().optional(),
  title: z.boolean().optional(),
  created_at: z.boolean().optional(),
  updated_at: z.boolean().optional(),
  deleted: z.boolean().optional(),
}).strict()


/////////////////////////////////////////
// INPUT TYPES
/////////////////////////////////////////

export const CardsWhereInputSchema: z.ZodType<Prisma.CardsWhereInput> = z.object({
  AND: z.union([ z.lazy(() => CardsWhereInputSchema),z.lazy(() => CardsWhereInputSchema).array() ]).optional(),
  OR: z.lazy(() => CardsWhereInputSchema).array().optional(),
  NOT: z.union([ z.lazy(() => CardsWhereInputSchema),z.lazy(() => CardsWhereInputSchema).array() ]).optional(),
  id: z.union([ z.lazy(() => UuidFilterSchema),z.string() ]).optional(),
  thread: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string() ]).optional().nullable(),
  prev_card: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string() ]).optional().nullable(),
  content: z.union([ z.lazy(() => StringNullableFilterSchema),z.string() ]).optional().nullable(),
  created_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
  updated_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
}).strict();

export const CardsOrderByWithRelationInputSchema: z.ZodType<Prisma.CardsOrderByWithRelationInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  thread: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  prev_card: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  content: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  created_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  updated_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
}).strict();

export const CardsWhereUniqueInputSchema: z.ZodType<Prisma.CardsWhereUniqueInput> = z.object({
  id: z.string().uuid()
})
.and(z.object({
  id: z.string().uuid().optional(),
  AND: z.union([ z.lazy(() => CardsWhereInputSchema),z.lazy(() => CardsWhereInputSchema).array() ]).optional(),
  OR: z.lazy(() => CardsWhereInputSchema).array().optional(),
  NOT: z.union([ z.lazy(() => CardsWhereInputSchema),z.lazy(() => CardsWhereInputSchema).array() ]).optional(),
  thread: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string().uuid() ]).optional().nullable(),
  prev_card: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string().uuid() ]).optional().nullable(),
  content: z.union([ z.lazy(() => StringNullableFilterSchema),z.string() ]).optional().nullable(),
  created_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
  updated_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
}).strict());

export const CardsOrderByWithAggregationInputSchema: z.ZodType<Prisma.CardsOrderByWithAggregationInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  thread: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  prev_card: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  content: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  created_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  updated_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  _count: z.lazy(() => CardsCountOrderByAggregateInputSchema).optional(),
  _max: z.lazy(() => CardsMaxOrderByAggregateInputSchema).optional(),
  _min: z.lazy(() => CardsMinOrderByAggregateInputSchema).optional()
}).strict();

export const CardsScalarWhereWithAggregatesInputSchema: z.ZodType<Prisma.CardsScalarWhereWithAggregatesInput> = z.object({
  AND: z.union([ z.lazy(() => CardsScalarWhereWithAggregatesInputSchema),z.lazy(() => CardsScalarWhereWithAggregatesInputSchema).array() ]).optional(),
  OR: z.lazy(() => CardsScalarWhereWithAggregatesInputSchema).array().optional(),
  NOT: z.union([ z.lazy(() => CardsScalarWhereWithAggregatesInputSchema),z.lazy(() => CardsScalarWhereWithAggregatesInputSchema).array() ]).optional(),
  id: z.union([ z.lazy(() => UuidWithAggregatesFilterSchema),z.string() ]).optional(),
  thread: z.union([ z.lazy(() => UuidNullableWithAggregatesFilterSchema),z.string() ]).optional().nullable(),
  prev_card: z.union([ z.lazy(() => UuidNullableWithAggregatesFilterSchema),z.string() ]).optional().nullable(),
  content: z.union([ z.lazy(() => StringNullableWithAggregatesFilterSchema),z.string() ]).optional().nullable(),
  created_at: z.union([ z.lazy(() => DateTimeNullableWithAggregatesFilterSchema),z.coerce.date() ]).optional().nullable(),
  updated_at: z.union([ z.lazy(() => DateTimeNullableWithAggregatesFilterSchema),z.coerce.date() ]).optional().nullable(),
}).strict();

export const ThreadsWhereInputSchema: z.ZodType<Prisma.ThreadsWhereInput> = z.object({
  AND: z.union([ z.lazy(() => ThreadsWhereInputSchema),z.lazy(() => ThreadsWhereInputSchema).array() ]).optional(),
  OR: z.lazy(() => ThreadsWhereInputSchema).array().optional(),
  NOT: z.union([ z.lazy(() => ThreadsWhereInputSchema),z.lazy(() => ThreadsWhereInputSchema).array() ]).optional(),
  id: z.union([ z.lazy(() => UuidFilterSchema),z.string() ]).optional(),
  parent_thread: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string() ]).optional().nullable(),
  prev_thread: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string() ]).optional().nullable(),
  title: z.union([ z.lazy(() => StringNullableFilterSchema),z.string() ]).optional().nullable(),
  created_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
  updated_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
  deleted: z.union([ z.lazy(() => BoolNullableFilterSchema),z.boolean() ]).optional().nullable(),
}).strict();

export const ThreadsOrderByWithRelationInputSchema: z.ZodType<Prisma.ThreadsOrderByWithRelationInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  parent_thread: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  prev_thread: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  title: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  created_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  updated_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  deleted: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
}).strict();

export const ThreadsWhereUniqueInputSchema: z.ZodType<Prisma.ThreadsWhereUniqueInput> = z.object({
  id: z.string().uuid()
})
.and(z.object({
  id: z.string().uuid().optional(),
  AND: z.union([ z.lazy(() => ThreadsWhereInputSchema),z.lazy(() => ThreadsWhereInputSchema).array() ]).optional(),
  OR: z.lazy(() => ThreadsWhereInputSchema).array().optional(),
  NOT: z.union([ z.lazy(() => ThreadsWhereInputSchema),z.lazy(() => ThreadsWhereInputSchema).array() ]).optional(),
  parent_thread: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string().uuid() ]).optional().nullable(),
  prev_thread: z.union([ z.lazy(() => UuidNullableFilterSchema),z.string().uuid() ]).optional().nullable(),
  title: z.union([ z.lazy(() => StringNullableFilterSchema),z.string() ]).optional().nullable(),
  created_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
  updated_at: z.union([ z.lazy(() => DateTimeNullableFilterSchema),z.coerce.date() ]).optional().nullable(),
  deleted: z.union([ z.lazy(() => BoolNullableFilterSchema),z.boolean() ]).optional().nullable(),
}).strict());

export const ThreadsOrderByWithAggregationInputSchema: z.ZodType<Prisma.ThreadsOrderByWithAggregationInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  parent_thread: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  prev_thread: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  title: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  created_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  updated_at: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  deleted: z.union([ z.lazy(() => SortOrderSchema),z.lazy(() => SortOrderInputSchema) ]).optional(),
  _count: z.lazy(() => ThreadsCountOrderByAggregateInputSchema).optional(),
  _max: z.lazy(() => ThreadsMaxOrderByAggregateInputSchema).optional(),
  _min: z.lazy(() => ThreadsMinOrderByAggregateInputSchema).optional()
}).strict();

export const ThreadsScalarWhereWithAggregatesInputSchema: z.ZodType<Prisma.ThreadsScalarWhereWithAggregatesInput> = z.object({
  AND: z.union([ z.lazy(() => ThreadsScalarWhereWithAggregatesInputSchema),z.lazy(() => ThreadsScalarWhereWithAggregatesInputSchema).array() ]).optional(),
  OR: z.lazy(() => ThreadsScalarWhereWithAggregatesInputSchema).array().optional(),
  NOT: z.union([ z.lazy(() => ThreadsScalarWhereWithAggregatesInputSchema),z.lazy(() => ThreadsScalarWhereWithAggregatesInputSchema).array() ]).optional(),
  id: z.union([ z.lazy(() => UuidWithAggregatesFilterSchema),z.string() ]).optional(),
  parent_thread: z.union([ z.lazy(() => UuidNullableWithAggregatesFilterSchema),z.string() ]).optional().nullable(),
  prev_thread: z.union([ z.lazy(() => UuidNullableWithAggregatesFilterSchema),z.string() ]).optional().nullable(),
  title: z.union([ z.lazy(() => StringNullableWithAggregatesFilterSchema),z.string() ]).optional().nullable(),
  created_at: z.union([ z.lazy(() => DateTimeNullableWithAggregatesFilterSchema),z.coerce.date() ]).optional().nullable(),
  updated_at: z.union([ z.lazy(() => DateTimeNullableWithAggregatesFilterSchema),z.coerce.date() ]).optional().nullable(),
  deleted: z.union([ z.lazy(() => BoolNullableWithAggregatesFilterSchema),z.boolean() ]).optional().nullable(),
}).strict();

export const CardsCreateInputSchema: z.ZodType<Prisma.CardsCreateInput> = z.object({
  id: z.string().uuid(),
  thread: z.string().uuid().optional().nullable(),
  prev_card: z.string().uuid().optional().nullable(),
  content: z.string().optional().nullable(),
  created_at: z.coerce.date().optional().nullable(),
  updated_at: z.coerce.date().optional().nullable()
}).strict();

export const CardsUncheckedCreateInputSchema: z.ZodType<Prisma.CardsUncheckedCreateInput> = z.object({
  id: z.string().uuid(),
  thread: z.string().uuid().optional().nullable(),
  prev_card: z.string().uuid().optional().nullable(),
  content: z.string().optional().nullable(),
  created_at: z.coerce.date().optional().nullable(),
  updated_at: z.coerce.date().optional().nullable()
}).strict();

export const CardsUpdateInputSchema: z.ZodType<Prisma.CardsUpdateInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_card: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  content: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const CardsUncheckedUpdateInputSchema: z.ZodType<Prisma.CardsUncheckedUpdateInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_card: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  content: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const CardsCreateManyInputSchema: z.ZodType<Prisma.CardsCreateManyInput> = z.object({
  id: z.string().uuid(),
  thread: z.string().uuid().optional().nullable(),
  prev_card: z.string().uuid().optional().nullable(),
  content: z.string().optional().nullable(),
  created_at: z.coerce.date().optional().nullable(),
  updated_at: z.coerce.date().optional().nullable()
}).strict();

export const CardsUpdateManyMutationInputSchema: z.ZodType<Prisma.CardsUpdateManyMutationInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_card: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  content: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const CardsUncheckedUpdateManyInputSchema: z.ZodType<Prisma.CardsUncheckedUpdateManyInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_card: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  content: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const ThreadsCreateInputSchema: z.ZodType<Prisma.ThreadsCreateInput> = z.object({
  id: z.string().uuid(),
  parent_thread: z.string().uuid().optional().nullable(),
  prev_thread: z.string().uuid().optional().nullable(),
  title: z.string().optional().nullable(),
  created_at: z.coerce.date().optional().nullable(),
  updated_at: z.coerce.date().optional().nullable(),
  deleted: z.boolean().optional().nullable()
}).strict();

export const ThreadsUncheckedCreateInputSchema: z.ZodType<Prisma.ThreadsUncheckedCreateInput> = z.object({
  id: z.string().uuid(),
  parent_thread: z.string().uuid().optional().nullable(),
  prev_thread: z.string().uuid().optional().nullable(),
  title: z.string().optional().nullable(),
  created_at: z.coerce.date().optional().nullable(),
  updated_at: z.coerce.date().optional().nullable(),
  deleted: z.boolean().optional().nullable()
}).strict();

export const ThreadsUpdateInputSchema: z.ZodType<Prisma.ThreadsUpdateInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  parent_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  title: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  deleted: z.union([ z.boolean(),z.lazy(() => NullableBoolFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const ThreadsUncheckedUpdateInputSchema: z.ZodType<Prisma.ThreadsUncheckedUpdateInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  parent_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  title: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  deleted: z.union([ z.boolean(),z.lazy(() => NullableBoolFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const ThreadsCreateManyInputSchema: z.ZodType<Prisma.ThreadsCreateManyInput> = z.object({
  id: z.string().uuid(),
  parent_thread: z.string().uuid().optional().nullable(),
  prev_thread: z.string().uuid().optional().nullable(),
  title: z.string().optional().nullable(),
  created_at: z.coerce.date().optional().nullable(),
  updated_at: z.coerce.date().optional().nullable(),
  deleted: z.boolean().optional().nullable()
}).strict();

export const ThreadsUpdateManyMutationInputSchema: z.ZodType<Prisma.ThreadsUpdateManyMutationInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  parent_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  title: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  deleted: z.union([ z.boolean(),z.lazy(() => NullableBoolFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const ThreadsUncheckedUpdateManyInputSchema: z.ZodType<Prisma.ThreadsUncheckedUpdateManyInput> = z.object({
  id: z.union([ z.string().uuid(),z.lazy(() => StringFieldUpdateOperationsInputSchema) ]).optional(),
  parent_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  prev_thread: z.union([ z.string().uuid(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  title: z.union([ z.string(),z.lazy(() => NullableStringFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  created_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  updated_at: z.union([ z.coerce.date(),z.lazy(() => NullableDateTimeFieldUpdateOperationsInputSchema) ]).optional().nullable(),
  deleted: z.union([ z.boolean(),z.lazy(() => NullableBoolFieldUpdateOperationsInputSchema) ]).optional().nullable(),
}).strict();

export const UuidFilterSchema: z.ZodType<Prisma.UuidFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  mode: z.lazy(() => QueryModeSchema).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidFilterSchema) ]).optional(),
}).strict();

export const UuidNullableFilterSchema: z.ZodType<Prisma.UuidNullableFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  mode: z.lazy(() => QueryModeSchema).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const StringNullableFilterSchema: z.ZodType<Prisma.StringNullableFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  contains: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  startsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  endsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  mode: z.lazy(() => QueryModeSchema).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedStringNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const DateTimeNullableFilterSchema: z.ZodType<Prisma.DateTimeNullableFilter> = z.object({
  equals: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  not: z.union([ z.coerce.date(),z.lazy(() => NestedDateTimeNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const SortOrderInputSchema: z.ZodType<Prisma.SortOrderInput> = z.object({
  sort: z.lazy(() => SortOrderSchema),
  nulls: z.lazy(() => NullsOrderSchema).optional()
}).strict();

export const CardsCountOrderByAggregateInputSchema: z.ZodType<Prisma.CardsCountOrderByAggregateInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  thread: z.lazy(() => SortOrderSchema).optional(),
  prev_card: z.lazy(() => SortOrderSchema).optional(),
  content: z.lazy(() => SortOrderSchema).optional(),
  created_at: z.lazy(() => SortOrderSchema).optional(),
  updated_at: z.lazy(() => SortOrderSchema).optional()
}).strict();

export const CardsMaxOrderByAggregateInputSchema: z.ZodType<Prisma.CardsMaxOrderByAggregateInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  thread: z.lazy(() => SortOrderSchema).optional(),
  prev_card: z.lazy(() => SortOrderSchema).optional(),
  content: z.lazy(() => SortOrderSchema).optional(),
  created_at: z.lazy(() => SortOrderSchema).optional(),
  updated_at: z.lazy(() => SortOrderSchema).optional()
}).strict();

export const CardsMinOrderByAggregateInputSchema: z.ZodType<Prisma.CardsMinOrderByAggregateInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  thread: z.lazy(() => SortOrderSchema).optional(),
  prev_card: z.lazy(() => SortOrderSchema).optional(),
  content: z.lazy(() => SortOrderSchema).optional(),
  created_at: z.lazy(() => SortOrderSchema).optional(),
  updated_at: z.lazy(() => SortOrderSchema).optional()
}).strict();

export const UuidWithAggregatesFilterSchema: z.ZodType<Prisma.UuidWithAggregatesFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  mode: z.lazy(() => QueryModeSchema).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidWithAggregatesFilterSchema) ]).optional(),
  _count: z.lazy(() => NestedIntFilterSchema).optional(),
  _min: z.lazy(() => NestedStringFilterSchema).optional(),
  _max: z.lazy(() => NestedStringFilterSchema).optional()
}).strict();

export const UuidNullableWithAggregatesFilterSchema: z.ZodType<Prisma.UuidNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  mode: z.lazy(() => QueryModeSchema).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedStringNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedStringNullableFilterSchema).optional()
}).strict();

export const StringNullableWithAggregatesFilterSchema: z.ZodType<Prisma.StringNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  contains: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  startsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  endsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  mode: z.lazy(() => QueryModeSchema).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedStringNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedStringNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedStringNullableFilterSchema).optional()
}).strict();

export const DateTimeNullableWithAggregatesFilterSchema: z.ZodType<Prisma.DateTimeNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  not: z.union([ z.coerce.date(),z.lazy(() => NestedDateTimeNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedDateTimeNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedDateTimeNullableFilterSchema).optional()
}).strict();

export const BoolNullableFilterSchema: z.ZodType<Prisma.BoolNullableFilter> = z.object({
  equals: z.union([ z.boolean(),z.lazy(() => BooleanFieldRefInputSchema) ]).optional().nullable(),
  not: z.union([ z.boolean(),z.lazy(() => NestedBoolNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const ThreadsCountOrderByAggregateInputSchema: z.ZodType<Prisma.ThreadsCountOrderByAggregateInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  parent_thread: z.lazy(() => SortOrderSchema).optional(),
  prev_thread: z.lazy(() => SortOrderSchema).optional(),
  title: z.lazy(() => SortOrderSchema).optional(),
  created_at: z.lazy(() => SortOrderSchema).optional(),
  updated_at: z.lazy(() => SortOrderSchema).optional(),
  deleted: z.lazy(() => SortOrderSchema).optional()
}).strict();

export const ThreadsMaxOrderByAggregateInputSchema: z.ZodType<Prisma.ThreadsMaxOrderByAggregateInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  parent_thread: z.lazy(() => SortOrderSchema).optional(),
  prev_thread: z.lazy(() => SortOrderSchema).optional(),
  title: z.lazy(() => SortOrderSchema).optional(),
  created_at: z.lazy(() => SortOrderSchema).optional(),
  updated_at: z.lazy(() => SortOrderSchema).optional(),
  deleted: z.lazy(() => SortOrderSchema).optional()
}).strict();

export const ThreadsMinOrderByAggregateInputSchema: z.ZodType<Prisma.ThreadsMinOrderByAggregateInput> = z.object({
  id: z.lazy(() => SortOrderSchema).optional(),
  parent_thread: z.lazy(() => SortOrderSchema).optional(),
  prev_thread: z.lazy(() => SortOrderSchema).optional(),
  title: z.lazy(() => SortOrderSchema).optional(),
  created_at: z.lazy(() => SortOrderSchema).optional(),
  updated_at: z.lazy(() => SortOrderSchema).optional(),
  deleted: z.lazy(() => SortOrderSchema).optional()
}).strict();

export const BoolNullableWithAggregatesFilterSchema: z.ZodType<Prisma.BoolNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.boolean(),z.lazy(() => BooleanFieldRefInputSchema) ]).optional().nullable(),
  not: z.union([ z.boolean(),z.lazy(() => NestedBoolNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedBoolNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedBoolNullableFilterSchema).optional()
}).strict();

export const StringFieldUpdateOperationsInputSchema: z.ZodType<Prisma.StringFieldUpdateOperationsInput> = z.object({
  set: z.string().optional()
}).strict();

export const NullableStringFieldUpdateOperationsInputSchema: z.ZodType<Prisma.NullableStringFieldUpdateOperationsInput> = z.object({
  set: z.string().optional().nullable()
}).strict();

export const NullableDateTimeFieldUpdateOperationsInputSchema: z.ZodType<Prisma.NullableDateTimeFieldUpdateOperationsInput> = z.object({
  set: z.coerce.date().optional().nullable()
}).strict();

export const NullableBoolFieldUpdateOperationsInputSchema: z.ZodType<Prisma.NullableBoolFieldUpdateOperationsInput> = z.object({
  set: z.boolean().optional().nullable()
}).strict();

export const NestedUuidFilterSchema: z.ZodType<Prisma.NestedUuidFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidFilterSchema) ]).optional(),
}).strict();

export const NestedUuidNullableFilterSchema: z.ZodType<Prisma.NestedUuidNullableFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const NestedStringNullableFilterSchema: z.ZodType<Prisma.NestedStringNullableFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  contains: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  startsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  endsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedStringNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const NestedDateTimeNullableFilterSchema: z.ZodType<Prisma.NestedDateTimeNullableFilter> = z.object({
  equals: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  not: z.union([ z.coerce.date(),z.lazy(() => NestedDateTimeNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const NestedUuidWithAggregatesFilterSchema: z.ZodType<Prisma.NestedUuidWithAggregatesFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidWithAggregatesFilterSchema) ]).optional(),
  _count: z.lazy(() => NestedIntFilterSchema).optional(),
  _min: z.lazy(() => NestedStringFilterSchema).optional(),
  _max: z.lazy(() => NestedStringFilterSchema).optional()
}).strict();

export const NestedIntFilterSchema: z.ZodType<Prisma.NestedIntFilter> = z.object({
  equals: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  in: z.union([ z.number().array(),z.lazy(() => ListIntFieldRefInputSchema) ]).optional(),
  notIn: z.union([ z.number().array(),z.lazy(() => ListIntFieldRefInputSchema) ]).optional(),
  lt: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  not: z.union([ z.number(),z.lazy(() => NestedIntFilterSchema) ]).optional(),
}).strict();

export const NestedStringFilterSchema: z.ZodType<Prisma.NestedStringFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  contains: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  startsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  endsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedStringFilterSchema) ]).optional(),
}).strict();

export const NestedUuidNullableWithAggregatesFilterSchema: z.ZodType<Prisma.NestedUuidNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedUuidNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedStringNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedStringNullableFilterSchema).optional()
}).strict();

export const NestedIntNullableFilterSchema: z.ZodType<Prisma.NestedIntNullableFilter> = z.object({
  equals: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.number().array(),z.lazy(() => ListIntFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.number().array(),z.lazy(() => ListIntFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.number(),z.lazy(() => IntFieldRefInputSchema) ]).optional(),
  not: z.union([ z.number(),z.lazy(() => NestedIntNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const NestedStringNullableWithAggregatesFilterSchema: z.ZodType<Prisma.NestedStringNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.string().array(),z.lazy(() => ListStringFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  contains: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  startsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  endsWith: z.union([ z.string(),z.lazy(() => StringFieldRefInputSchema) ]).optional(),
  not: z.union([ z.string(),z.lazy(() => NestedStringNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedStringNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedStringNullableFilterSchema).optional()
}).strict();

export const NestedDateTimeNullableWithAggregatesFilterSchema: z.ZodType<Prisma.NestedDateTimeNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional().nullable(),
  in: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  notIn: z.union([ z.coerce.date().array(),z.lazy(() => ListDateTimeFieldRefInputSchema) ]).optional().nullable(),
  lt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  lte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gt: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  gte: z.union([ z.coerce.date(),z.lazy(() => DateTimeFieldRefInputSchema) ]).optional(),
  not: z.union([ z.coerce.date(),z.lazy(() => NestedDateTimeNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedDateTimeNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedDateTimeNullableFilterSchema).optional()
}).strict();

export const NestedBoolNullableFilterSchema: z.ZodType<Prisma.NestedBoolNullableFilter> = z.object({
  equals: z.union([ z.boolean(),z.lazy(() => BooleanFieldRefInputSchema) ]).optional().nullable(),
  not: z.union([ z.boolean(),z.lazy(() => NestedBoolNullableFilterSchema) ]).optional().nullable(),
}).strict();

export const NestedBoolNullableWithAggregatesFilterSchema: z.ZodType<Prisma.NestedBoolNullableWithAggregatesFilter> = z.object({
  equals: z.union([ z.boolean(),z.lazy(() => BooleanFieldRefInputSchema) ]).optional().nullable(),
  not: z.union([ z.boolean(),z.lazy(() => NestedBoolNullableWithAggregatesFilterSchema) ]).optional().nullable(),
  _count: z.lazy(() => NestedIntNullableFilterSchema).optional(),
  _min: z.lazy(() => NestedBoolNullableFilterSchema).optional(),
  _max: z.lazy(() => NestedBoolNullableFilterSchema).optional()
}).strict();

/////////////////////////////////////////
// ARGS
/////////////////////////////////////////

export const CardsFindFirstArgsSchema: z.ZodType<Prisma.CardsFindFirstArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereInputSchema.optional(),
  orderBy: z.union([ CardsOrderByWithRelationInputSchema.array(),CardsOrderByWithRelationInputSchema ]).optional(),
  cursor: CardsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
  distinct: z.union([ CardsScalarFieldEnumSchema,CardsScalarFieldEnumSchema.array() ]).optional(),
}).strict() 

export const CardsFindFirstOrThrowArgsSchema: z.ZodType<Prisma.CardsFindFirstOrThrowArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereInputSchema.optional(),
  orderBy: z.union([ CardsOrderByWithRelationInputSchema.array(),CardsOrderByWithRelationInputSchema ]).optional(),
  cursor: CardsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
  distinct: z.union([ CardsScalarFieldEnumSchema,CardsScalarFieldEnumSchema.array() ]).optional(),
}).strict() 

export const CardsFindManyArgsSchema: z.ZodType<Prisma.CardsFindManyArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereInputSchema.optional(),
  orderBy: z.union([ CardsOrderByWithRelationInputSchema.array(),CardsOrderByWithRelationInputSchema ]).optional(),
  cursor: CardsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
  distinct: z.union([ CardsScalarFieldEnumSchema,CardsScalarFieldEnumSchema.array() ]).optional(),
}).strict() 

export const CardsAggregateArgsSchema: z.ZodType<Prisma.CardsAggregateArgs> = z.object({
  where: CardsWhereInputSchema.optional(),
  orderBy: z.union([ CardsOrderByWithRelationInputSchema.array(),CardsOrderByWithRelationInputSchema ]).optional(),
  cursor: CardsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
}).strict() 

export const CardsGroupByArgsSchema: z.ZodType<Prisma.CardsGroupByArgs> = z.object({
  where: CardsWhereInputSchema.optional(),
  orderBy: z.union([ CardsOrderByWithAggregationInputSchema.array(),CardsOrderByWithAggregationInputSchema ]).optional(),
  by: CardsScalarFieldEnumSchema.array(),
  having: CardsScalarWhereWithAggregatesInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
}).strict() 

export const CardsFindUniqueArgsSchema: z.ZodType<Prisma.CardsFindUniqueArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereUniqueInputSchema,
}).strict() 

export const CardsFindUniqueOrThrowArgsSchema: z.ZodType<Prisma.CardsFindUniqueOrThrowArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereUniqueInputSchema,
}).strict() 

export const ThreadsFindFirstArgsSchema: z.ZodType<Prisma.ThreadsFindFirstArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereInputSchema.optional(),
  orderBy: z.union([ ThreadsOrderByWithRelationInputSchema.array(),ThreadsOrderByWithRelationInputSchema ]).optional(),
  cursor: ThreadsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
  distinct: z.union([ ThreadsScalarFieldEnumSchema,ThreadsScalarFieldEnumSchema.array() ]).optional(),
}).strict() 

export const ThreadsFindFirstOrThrowArgsSchema: z.ZodType<Prisma.ThreadsFindFirstOrThrowArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereInputSchema.optional(),
  orderBy: z.union([ ThreadsOrderByWithRelationInputSchema.array(),ThreadsOrderByWithRelationInputSchema ]).optional(),
  cursor: ThreadsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
  distinct: z.union([ ThreadsScalarFieldEnumSchema,ThreadsScalarFieldEnumSchema.array() ]).optional(),
}).strict() 

export const ThreadsFindManyArgsSchema: z.ZodType<Prisma.ThreadsFindManyArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereInputSchema.optional(),
  orderBy: z.union([ ThreadsOrderByWithRelationInputSchema.array(),ThreadsOrderByWithRelationInputSchema ]).optional(),
  cursor: ThreadsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
  distinct: z.union([ ThreadsScalarFieldEnumSchema,ThreadsScalarFieldEnumSchema.array() ]).optional(),
}).strict() 

export const ThreadsAggregateArgsSchema: z.ZodType<Prisma.ThreadsAggregateArgs> = z.object({
  where: ThreadsWhereInputSchema.optional(),
  orderBy: z.union([ ThreadsOrderByWithRelationInputSchema.array(),ThreadsOrderByWithRelationInputSchema ]).optional(),
  cursor: ThreadsWhereUniqueInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
}).strict() 

export const ThreadsGroupByArgsSchema: z.ZodType<Prisma.ThreadsGroupByArgs> = z.object({
  where: ThreadsWhereInputSchema.optional(),
  orderBy: z.union([ ThreadsOrderByWithAggregationInputSchema.array(),ThreadsOrderByWithAggregationInputSchema ]).optional(),
  by: ThreadsScalarFieldEnumSchema.array(),
  having: ThreadsScalarWhereWithAggregatesInputSchema.optional(),
  take: z.number().optional(),
  skip: z.number().optional(),
}).strict() 

export const ThreadsFindUniqueArgsSchema: z.ZodType<Prisma.ThreadsFindUniqueArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereUniqueInputSchema,
}).strict() 

export const ThreadsFindUniqueOrThrowArgsSchema: z.ZodType<Prisma.ThreadsFindUniqueOrThrowArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereUniqueInputSchema,
}).strict() 

export const CardsCreateArgsSchema: z.ZodType<Prisma.CardsCreateArgs> = z.object({
  select: CardsSelectSchema.optional(),
  data: z.union([ CardsCreateInputSchema,CardsUncheckedCreateInputSchema ]),
}).strict() 

export const CardsUpsertArgsSchema: z.ZodType<Prisma.CardsUpsertArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereUniqueInputSchema,
  create: z.union([ CardsCreateInputSchema,CardsUncheckedCreateInputSchema ]),
  update: z.union([ CardsUpdateInputSchema,CardsUncheckedUpdateInputSchema ]),
}).strict() 

export const CardsCreateManyArgsSchema: z.ZodType<Prisma.CardsCreateManyArgs> = z.object({
  data: z.union([ CardsCreateManyInputSchema,CardsCreateManyInputSchema.array() ]),
  skipDuplicates: z.boolean().optional(),
}).strict() 

export const CardsAndReturnCreateManyArgsSchema: z.ZodType<Prisma.CardsAndReturnCreateManyArgs> = z.object({
  data: z.union([ CardsCreateManyInputSchema,CardsCreateManyInputSchema.array() ]),
  skipDuplicates: z.boolean().optional(),
}).strict() 

export const CardsDeleteArgsSchema: z.ZodType<Prisma.CardsDeleteArgs> = z.object({
  select: CardsSelectSchema.optional(),
  where: CardsWhereUniqueInputSchema,
}).strict() 

export const CardsUpdateArgsSchema: z.ZodType<Prisma.CardsUpdateArgs> = z.object({
  select: CardsSelectSchema.optional(),
  data: z.union([ CardsUpdateInputSchema,CardsUncheckedUpdateInputSchema ]),
  where: CardsWhereUniqueInputSchema,
}).strict() 

export const CardsUpdateManyArgsSchema: z.ZodType<Prisma.CardsUpdateManyArgs> = z.object({
  data: z.union([ CardsUpdateManyMutationInputSchema,CardsUncheckedUpdateManyInputSchema ]),
  where: CardsWhereInputSchema.optional(),
}).strict() 

export const CardsDeleteManyArgsSchema: z.ZodType<Prisma.CardsDeleteManyArgs> = z.object({
  where: CardsWhereInputSchema.optional(),
}).strict() 

export const ThreadsCreateArgsSchema: z.ZodType<Prisma.ThreadsCreateArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  data: z.union([ ThreadsCreateInputSchema,ThreadsUncheckedCreateInputSchema ]),
}).strict() 

export const ThreadsUpsertArgsSchema: z.ZodType<Prisma.ThreadsUpsertArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereUniqueInputSchema,
  create: z.union([ ThreadsCreateInputSchema,ThreadsUncheckedCreateInputSchema ]),
  update: z.union([ ThreadsUpdateInputSchema,ThreadsUncheckedUpdateInputSchema ]),
}).strict() 

export const ThreadsCreateManyArgsSchema: z.ZodType<Prisma.ThreadsCreateManyArgs> = z.object({
  data: z.union([ ThreadsCreateManyInputSchema,ThreadsCreateManyInputSchema.array() ]),
  skipDuplicates: z.boolean().optional(),
}).strict() 

export const ThreadsAndReturnCreateManyArgsSchema: z.ZodType<Prisma.ThreadsAndReturnCreateManyArgs> = z.object({
  data: z.union([ ThreadsCreateManyInputSchema,ThreadsCreateManyInputSchema.array() ]),
  skipDuplicates: z.boolean().optional(),
}).strict() 

export const ThreadsDeleteArgsSchema: z.ZodType<Prisma.ThreadsDeleteArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  where: ThreadsWhereUniqueInputSchema,
}).strict() 

export const ThreadsUpdateArgsSchema: z.ZodType<Prisma.ThreadsUpdateArgs> = z.object({
  select: ThreadsSelectSchema.optional(),
  data: z.union([ ThreadsUpdateInputSchema,ThreadsUncheckedUpdateInputSchema ]),
  where: ThreadsWhereUniqueInputSchema,
}).strict() 

export const ThreadsUpdateManyArgsSchema: z.ZodType<Prisma.ThreadsUpdateManyArgs> = z.object({
  data: z.union([ ThreadsUpdateManyMutationInputSchema,ThreadsUncheckedUpdateManyInputSchema ]),
  where: ThreadsWhereInputSchema.optional(),
}).strict() 

export const ThreadsDeleteManyArgsSchema: z.ZodType<Prisma.ThreadsDeleteManyArgs> = z.object({
  where: ThreadsWhereInputSchema.optional(),
}).strict() 

interface CardsGetPayload extends HKT {
  readonly _A?: boolean | null | undefined | Prisma.CardsArgs
  readonly type: Omit<Prisma.CardsGetPayload<this['_A']>, "Please either choose `select` or `include`">
}

interface ThreadsGetPayload extends HKT {
  readonly _A?: boolean | null | undefined | Prisma.ThreadsArgs
  readonly type: Omit<Prisma.ThreadsGetPayload<this['_A']>, "Please either choose `select` or `include`">
}

export const tableSchemas = {
  cards: {
    fields: new Map([
      [
        "id",
        "UUID"
      ],
      [
        "thread",
        "UUID"
      ],
      [
        "prev_card",
        "UUID"
      ],
      [
        "content",
        "TEXT"
      ],
      [
        "created_at",
        "TIMESTAMP"
      ],
      [
        "updated_at",
        "TIMESTAMP"
      ]
    ]),
    relations: [
    ],
    modelSchema: (CardsCreateInputSchema as any)
      .partial()
      .or((CardsUncheckedCreateInputSchema as any).partial()),
    createSchema: CardsCreateArgsSchema,
    createManySchema: CardsCreateManyArgsSchema,
    findUniqueSchema: CardsFindUniqueArgsSchema,
    findSchema: CardsFindFirstArgsSchema,
    updateSchema: CardsUpdateArgsSchema,
    updateManySchema: CardsUpdateManyArgsSchema,
    upsertSchema: CardsUpsertArgsSchema,
    deleteSchema: CardsDeleteArgsSchema,
    deleteManySchema: CardsDeleteManyArgsSchema
  } as TableSchema<
    z.infer<typeof CardsUncheckedCreateInputSchema>,
    Prisma.CardsCreateArgs['data'],
    Prisma.CardsUpdateArgs['data'],
    Prisma.CardsFindFirstArgs['select'],
    Prisma.CardsFindFirstArgs['where'],
    Prisma.CardsFindUniqueArgs['where'],
    never,
    Prisma.CardsFindFirstArgs['orderBy'],
    Prisma.CardsScalarFieldEnum,
    CardsGetPayload
  >,
  threads: {
    fields: new Map([
      [
        "id",
        "UUID"
      ],
      [
        "parent_thread",
        "UUID"
      ],
      [
        "prev_thread",
        "UUID"
      ],
      [
        "title",
        "TEXT"
      ],
      [
        "created_at",
        "TIMESTAMP"
      ],
      [
        "updated_at",
        "TIMESTAMP"
      ],
      [
        "deleted",
        "BOOL"
      ]
    ]),
    relations: [
    ],
    modelSchema: (ThreadsCreateInputSchema as any)
      .partial()
      .or((ThreadsUncheckedCreateInputSchema as any).partial()),
    createSchema: ThreadsCreateArgsSchema,
    createManySchema: ThreadsCreateManyArgsSchema,
    findUniqueSchema: ThreadsFindUniqueArgsSchema,
    findSchema: ThreadsFindFirstArgsSchema,
    updateSchema: ThreadsUpdateArgsSchema,
    updateManySchema: ThreadsUpdateManyArgsSchema,
    upsertSchema: ThreadsUpsertArgsSchema,
    deleteSchema: ThreadsDeleteArgsSchema,
    deleteManySchema: ThreadsDeleteManyArgsSchema
  } as TableSchema<
    z.infer<typeof ThreadsUncheckedCreateInputSchema>,
    Prisma.ThreadsCreateArgs['data'],
    Prisma.ThreadsUpdateArgs['data'],
    Prisma.ThreadsFindFirstArgs['select'],
    Prisma.ThreadsFindFirstArgs['where'],
    Prisma.ThreadsFindUniqueArgs['where'],
    never,
    Prisma.ThreadsFindFirstArgs['orderBy'],
    Prisma.ThreadsScalarFieldEnum,
    ThreadsGetPayload
  >,
}

export const schema = new DbSchema(tableSchemas, migrations, pgMigrations)
export type Electric = ElectricClient<typeof schema>
