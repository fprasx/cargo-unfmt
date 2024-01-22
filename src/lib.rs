use proc_macro2::Span;
use syn::{
    visit::Visit, Abi, AngleBracketedGenericArguments, Arm, AssocConst, AssocType, AttrStyle,
    Attribute, BareFnArg, BareVariadic, BinOp, Block, BoundLifetimes, ConstParam, Constraint, Data,
    DataEnum, DataStruct, DataUnion, DeriveInput, Expr, ExprArray, ExprAssign, ExprAsync,
    ExprAwait, ExprBinary, ExprBlock, ExprBreak, ExprCall, ExprCast, ExprClosure, ExprConst,
    ExprContinue, ExprField, ExprForLoop, ExprGroup, ExprIf, ExprIndex, ExprInfer, ExprLet,
    ExprLit, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall, ExprParen, ExprPath, ExprRange,
    ExprReference, ExprRepeat, ExprReturn, ExprStruct, ExprTry, ExprTryBlock, ExprTuple, ExprUnary,
    ExprUnsafe, ExprWhile, ExprYield, Field, FieldMutability, FieldPat, FieldValue, Fields,
    FieldsNamed, FieldsUnnamed, File, FnArg, ForeignItem, ForeignItemFn, ForeignItemMacro,
    ForeignItemStatic, ForeignItemType, GenericArgument, GenericParam, Generics, Ident, ImplItem,
    ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, ImplRestriction, Index, Item,
    ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Label,
    Lifetime, LifetimeParam, Lit, LitBool, LitByte, LitByteStr, LitChar, LitFloat, LitInt, LitStr,
    Local, LocalInit, Macro, MacroDelimiter, Member, Meta, MetaList, MetaNameValue,
    ParenthesizedGenericArguments, Pat, PatIdent, PatOr, PatParen, PatReference, PatRest, PatSlice,
    PatStruct, PatTuple, PatTupleStruct, PatType, PatWild, Path, PathArguments, PathSegment,
    PredicateLifetime, PredicateType, QSelf, RangeLimits, Receiver, ReturnType, Signature,
    StaticMutability, Stmt, StmtMacro, TraitBound, TraitBoundModifier, TraitItem, TraitItemConst,
    TraitItemFn, TraitItemMacro, TraitItemType, Type, TypeArray, TypeBareFn, TypeGroup,
    TypeImplTrait, TypeInfer, TypeMacro, TypeNever, TypeParam, TypeParamBound, TypeParen, TypePath,
    TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple, UnOp, UseGlob, UseGroup,
    UseName, UsePath, UseRename, UseTree, Variadic, Variant, VisRestricted, Visibility,
    WhereClause, WherePredicate,
};

enum Morpheme {
    Repel(Box<str>),
    RepelColon(Box<str>),
    Tight(Box<str>),
    Spacer,
}

struct TokenVisitor {
    tokens: Vec<Morpheme>,
}

impl TokenVisitor {
    fn repel(&mut self, s: impl Into<Box<str>>) {
        self.tokens.push(Morpheme::Repel(s.into()))
    }

    fn repel_colon(&mut self, s: impl Into<Box<str>>) {
        self.tokens.push(Morpheme::RepelColon(s.into()))
    }

    fn tight(&mut self, s: impl Into<Box<str>>) {
        self.tokens.push(Morpheme::Tight(s.into()))
    }

    fn spacer(&mut self) {
        self.tokens.push(Morpheme::Spacer)
    }
}

impl Visit<'_> for TokenVisitor {
    fn visit_abi(&mut self, Abi { name, .. }: &'_ Abi) {
        self.repel("extern");
        if let Some(name) = name {
            self.repel(format!("\"{}\"", name.value()));
        }
    }

    fn visit_angle_bracketed_generic_arguments(
        &mut self,
        AngleBracketedGenericArguments {
            colon2_token, args, ..
        }: &'_ AngleBracketedGenericArguments,
    ) {
        if colon2_token.is_some() {
            self.repel_colon("::");
        }
        self.tight("<");
        let trailing = args.trailing_punct();
        for arg in args {
            self.visit_generic_argument(arg);
        }
        if trailing {
            self.tight(",");
        }
        self.tight(">");
    }

    fn visit_arm(
        &mut self,
        Arm {
            attrs,
            pat,
            guard,
            body,
            ..
        }: &'_ Arm,
    ) {
        self.visit_pat(pat);
        if let Some((_if, expr)) = guard {
            self.repel("if");
            self.visit_expr(expr)
        }
        self.tight("=>");
        self.visit_expr(body);
        self.tight(",");
    }

    fn visit_assoc_const(
        &mut self,
        AssocConst {
            ident,
            generics,
            value,
            ..
        }: &'_ AssocConst,
    ) {
        self.visit_ident(ident);
        if let Some(generics) = generics {
            self.visit_angle_bracketed_generic_arguments(generics);
        }
        self.tight("=");
        self.visit_expr(value);
    }

    fn visit_assoc_type(
        &mut self,
        AssocType {
            ident,
            generics,
            ty,
            ..
        }: &'_ AssocType,
    ) {
        self.visit_ident(ident);
        if let Some(generics) = generics {
            self.visit_angle_bracketed_generic_arguments(generics);
        }
        self.tight("=");
        self.visit_type(ty);
    }

    fn visit_attr_style(&mut self, style: &'_ AttrStyle) {
        match style {
            AttrStyle::Outer => (),
            AttrStyle::Inner(_) => self.tight("!"),
        }
    }

    fn visit_attribute(&mut self, Attribute { style, meta, .. }: &'_ Attribute) {
        self.tight("#");
        self.visit_attr_style(style);
        self.tight("[");
        self.visit_meta(meta); // hi zuck
        self.tight("]");
    }

    fn visit_bare_fn_arg(&mut self, BareFnArg { attrs, name, ty }: &'_ BareFnArg) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        if let Some((ident, _)) = name {
            self.visit_ident(ident);
            self.tight(":");
        }
        self.visit_type(ty);
    }

    fn visit_bare_variadic(
        &mut self,
        BareVariadic {
            attrs, name, comma, ..
        }: &'_ BareVariadic,
    ) {
        for attr in attrs {
            self.visit_attribute(attr);
        }
        if let Some((ident, _)) = name {
            self.visit_ident(ident);
            self.tight(":");
        }
        self.tight("...");
        if comma.is_some() {
            self.tight(",");
        }
    }

    fn visit_bin_op(&mut self, b: &'_ BinOp) {
        match b {
            BinOp::Add(_) => self.tight("+"),
            BinOp::Sub(_) => self.tight("-"),
            BinOp::Mul(_) => self.tight("*"),
            BinOp::Div(_) => self.tight("/"),
            BinOp::Rem(_) => self.tight("%"),
            BinOp::And(_) => self.tight("&&"),
            BinOp::Or(_) => self.tight("||"),
            BinOp::BitXor(_) => self.tight("^"),
            BinOp::BitAnd(_) => self.tight("&"),
            BinOp::BitOr(_) => self.tight("|"),
            BinOp::Shl(_) => self.tight("<<"),
            BinOp::Shr(_) => self.tight(">>"),
            BinOp::Eq(_) => self.tight("=="),
            BinOp::Lt(_) => self.tight("<"),
            BinOp::Le(_) => self.tight("<="),
            BinOp::Ne(_) => self.tight("!="),
            BinOp::Ge(_) => self.tight(">"),
            BinOp::Gt(_) => self.tight(">="),
            BinOp::AddAssign(_) => self.tight("+="),
            BinOp::SubAssign(_) => self.tight("-="),
            BinOp::MulAssign(_) => self.tight("*="),
            BinOp::DivAssign(_) => self.tight("/="),
            BinOp::RemAssign(_) => self.tight("%="),
            BinOp::BitXorAssign(_) => self.tight("^="),
            BinOp::BitAndAssign(_) => self.tight("&="),
            BinOp::BitOrAssign(_) => self.tight("|="),
            BinOp::ShlAssign(_) => self.tight("<<="),
            BinOp::ShrAssign(_) => self.tight(">>="),
            _ => panic!("unknown binary operation"),
        }
    }

    fn visit_block(&mut self, Block { stmts, .. }: &'_ Block) {
        self.tight("{");
        for stmt in stmts {
            self.visit_stmt(stmt);
            self.spacer();
        }
        self.tight("}");
        self.spacer();
    }

    fn visit_bound_lifetimes(&mut self, BoundLifetimes { lifetimes, .. }: &'_ BoundLifetimes) {
        self.repel("for");
        self.tight("<");
        let trailing = lifetimes.trailing_punct();
        for gp in lifetimes {
            self.visit_generic_param(gp);
        }
        if trailing {
            self.tight(",");
        }
        self.tight(">");
    }

    fn visit_const_param(
        &mut self,
        ConstParam {
            attrs,
            ident,
            ty,
            default,
            ..
        }: &'_ ConstParam,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.repel("const");
        self.visit_ident(ident);
        self.tight(":");
        self.visit_type(ty);
        self.tight("=");
        if let Some(e) = default {
            self.visit_expr(e);
        }
    }

    fn visit_constraint(
        &mut self,
        Constraint {
            ident,
            generics,
            bounds,
            ..
        }: &'_ Constraint,
    ) {
        self.visit_ident(ident);
        if let Some(generics) = generics {
            self.visit_angle_bracketed_generic_arguments(generics);
        }
        self.tight(":");
        let trailing = bounds.trailing_punct();
        for bound in bounds {
            self.visit_type_param_bound(bound);
        }
        if trailing {
            self.tight("+");
        }
    }

    fn visit_data(&mut self, data: &'_ Data) {
        match data {
            Data::Struct(s) => self.visit_data_struct(s),
            Data::Enum(e) => self.visit_data_enum(e),
            Data::Union(u) => self.visit_data_union(u),
        }
    }

    fn visit_data_enum(&mut self, DataEnum { variants, .. }: &'_ DataEnum) {
        self.repel("enum");
        self.tight("{");
        let trailing = variants.trailing_punct();
        for variant in variants {
            self.visit_variant(variant)
        }
        if trailing {
            self.tight(",");
        }
        self.tight("}");
    }

    fn visit_data_struct(
        &mut self,
        DataStruct {
            fields, semi_token, ..
        }: &'_ DataStruct,
    ) {
        self.repel("struct");
        self.visit_fields(fields);
        if semi_token.is_some() {
            self.tight(";");
        }
    }

    fn visit_data_union(&mut self, DataUnion { fields, .. }: &'_ DataUnion) {
        self.repel("union");
        self.visit_fields_named(fields);
    }

    fn visit_derive_input(
        &mut self,
        DeriveInput {
            attrs,
            vis,
            ident,
            generics,
            data,
        }: &'_ DeriveInput,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.visit_visibility(vis);
        self.visit_ident(ident);
        self.visit_generics(generics);
        self.visit_data(data);
    }

    fn visit_expr(&mut self, expr: &'_ Expr) {
        match expr {
            Expr::Array(e) => self.visit_expr_array(e),
            Expr::Assign(e) => self.visit_expr_assign(e),
            Expr::Async(e) => self.visit_expr_async(e),
            Expr::Await(e) => self.visit_expr_await(e),
            Expr::Binary(e) => self.visit_expr_binary(e),
            Expr::Block(e) => self.visit_expr_block(e),
            Expr::Break(e) => self.visit_expr_break(e),
            Expr::Call(e) => self.visit_expr_call(e),
            Expr::Cast(e) => self.visit_expr_cast(e),
            Expr::Closure(e) => self.visit_expr_closure(e),
            Expr::Const(e) => self.visit_expr_const(e),
            Expr::Continue(e) => self.visit_expr_continue(e),
            Expr::Field(e) => self.visit_expr_field(e),
            Expr::ForLoop(e) => self.visit_expr_for_loop(e),
            Expr::Group(e) => self.visit_expr_group(e),
            Expr::If(e) => self.visit_expr_if(e),
            Expr::Index(e) => self.visit_expr_index(e),
            Expr::Infer(e) => self.visit_expr_infer(e),
            Expr::Let(e) => self.visit_expr_let(e),
            Expr::Lit(e) => self.visit_expr_lit(e),
            Expr::Loop(e) => self.visit_expr_loop(e),
            Expr::Macro(e) => self.visit_expr_macro(e),
            Expr::Match(e) => self.visit_expr_match(e),
            Expr::MethodCall(e) => self.visit_expr_method_call(e),
            Expr::Paren(e) => self.visit_expr_paren(e),
            Expr::Path(e) => self.visit_expr_path(e),
            Expr::Range(e) => self.visit_expr_range(e),
            Expr::Reference(e) => self.visit_expr_reference(e),
            Expr::Repeat(e) => self.visit_expr_repeat(e),
            Expr::Return(e) => self.visit_expr_return(e),
            Expr::Struct(e) => self.visit_expr_struct(e),
            Expr::Try(e) => self.visit_expr_try(e),
            Expr::TryBlock(e) => self.visit_expr_try_block(e),
            Expr::Tuple(e) => self.visit_expr_tuple(e),
            Expr::Unary(e) => self.visit_expr_unary(e),
            Expr::Unsafe(e) => self.visit_expr_unsafe(e),
            Expr::Verbatim(e) => panic!("what the heck is verbatim: {e}"),
            Expr::While(e) => self.visit_expr_while(e),
            Expr::Yield(e) => self.visit_expr_yield(e),
            _ => todo!(),
        }
    }

    fn visit_expr_array(&mut self, ExprArray { attrs, elems, .. }: &'_ ExprArray) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.tight("[");
        let trailing = elems.trailing_punct();
        for expr in elems {
            self.visit_expr(expr);
        }
        if trailing {
            self.tight(",");
        }
        self.tight("]");
    }

    fn visit_expr_assign(
        &mut self,
        ExprAssign {
            attrs, left, right, ..
        }: &'_ ExprAssign,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.visit_expr(left);
        self.tight("=");
        self.visit_expr(right);
    }

    fn visit_expr_async(
        &mut self,
        ExprAsync {
            attrs,
            capture,
            block,
            ..
        }: &'_ ExprAsync,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.repel("async");
        if capture.is_some() {
            self.repel("move");
        }
        self.visit_block(block)
    }

    fn visit_expr_await(&mut self, ExprAwait { attrs, base, .. }: &'_ ExprAwait) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.visit_expr(base);
        self.tight(".");
        self.repel("await");
    }

    fn visit_expr_binary(
        &mut self,
        ExprBinary {
            attrs,
            left,
            op,
            right,
        }: &'_ ExprBinary,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.visit_expr(left);
        self.visit_bin_op(op);
        self.visit_expr(right);
    }

    fn visit_expr_block(
        &mut self,
        ExprBlock {
            attrs,
            label,
            block,
        }: &'_ ExprBlock,
    ) {
        for attr in attrs {
            self.visit_attribute(attr);
        }
        if let Some(Label { name, .. }) = label {
            self.visit_lifetime(name);
            self.tight(",");
        }
        self.visit_block(block);
    }

    fn visit_expr_break(
        &mut self,
        ExprBreak {
            attrs, label, expr, ..
        }: &'_ ExprBreak,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.repel("break");
        if let Some(label) = label {
            self.visit_lifetime(label)
        }
        if let Some(expr) = expr {
            self.visit_expr(expr)
        }
    }

    fn visit_expr_call(
        &mut self,
        ExprCall {
            attrs, func, args, ..
        }: &'_ ExprCall,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.visit_expr(func);
        self.tight("(");
        let trailing = args.trailing_punct();
        for arg in args {
            self.visit_expr(arg);
        }
        if trailing {
            self.tight(",")
        }
        self.tight(")");
    }

    fn visit_expr_cast(
        &mut self,
        ExprCast {
            attrs,
            expr,
            as_token,
            ty,
        }: &'_ ExprCast,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        self.visit_expr(expr);
        self.repel("as");
        self.visit_type(ty)
    }

    fn visit_expr_closure(
        &mut self,
        ExprClosure {
            attrs,
            lifetimes,
            constness,
            movability,
            asyncness,
            capture,
            inputs,
            output,
            body,
            ..
        }: &'_ ExprClosure,
    ) {
        for attr in attrs {
            self.visit_attribute(attr)
        }
        if let Some(lt) = lifetimes {
            self.visit_bound_lifetimes(lt)
        }
        if constness.is_some() {
            self.repel("const")
        }
        if movability.is_some() {
            self.repel("move")
        }
        if asyncness.is_some() {
            self.repel("async")
        }
        if capture.is_some() {
            self.repel("move")
        }
        self.tight("|");
        let trailing = inputs.trailing_punct();
        for pat in inputs {
            self.visit_pat(pat);
        }
        if trailing {
            self.tight(",")
        }
        self.tight("|");
        self.visit_return_type(output);
        self.visit_expr(body)
    }

    fn visit_expr_const(&mut self, i: &'_ ExprConst) {
        todo!()
    }

    fn visit_expr_continue(&mut self, i: &'_ ExprContinue) {
        todo!()
    }

    fn visit_expr_field(&mut self, i: &'_ ExprField) {
        todo!()
    }

    fn visit_expr_for_loop(&mut self, i: &'_ ExprForLoop) {
        todo!()
    }

    fn visit_expr_group(&mut self, i: &'_ ExprGroup) {
        todo!()
    }

    fn visit_expr_if(&mut self, i: &'_ ExprIf) {
        todo!()
    }

    fn visit_expr_index(&mut self, i: &'_ ExprIndex) {
        todo!()
    }

    fn visit_expr_infer(&mut self, i: &'_ ExprInfer) {
        todo!()
    }

    fn visit_expr_let(&mut self, i: &'_ ExprLet) {
        todo!()
    }

    fn visit_expr_lit(&mut self, i: &'_ ExprLit) {
        todo!()
    }

    fn visit_expr_loop(&mut self, i: &'_ ExprLoop) {
        todo!()
    }

    fn visit_expr_macro(&mut self, i: &'_ ExprMacro) {
        todo!()
    }

    fn visit_expr_match(&mut self, i: &'_ ExprMatch) {
        todo!()
    }

    fn visit_expr_method_call(&mut self, i: &'_ ExprMethodCall) {
        todo!()
    }

    fn visit_expr_paren(&mut self, i: &'_ ExprParen) {
        todo!()
    }

    fn visit_expr_path(&mut self, i: &'_ ExprPath) {
        todo!()
    }

    fn visit_expr_range(&mut self, i: &'_ ExprRange) {
        todo!()
    }

    fn visit_expr_reference(&mut self, i: &'_ ExprReference) {
        todo!()
    }

    fn visit_expr_repeat(&mut self, i: &'_ ExprRepeat) {
        todo!()
    }

    fn visit_expr_return(&mut self, i: &'_ ExprReturn) {
        todo!()
    }

    fn visit_expr_struct(&mut self, i: &'_ ExprStruct) {
        todo!()
    }

    fn visit_expr_try(&mut self, i: &'_ ExprTry) {
        todo!()
    }

    fn visit_expr_try_block(&mut self, i: &'_ ExprTryBlock) {
        todo!()
    }

    fn visit_expr_tuple(&mut self, i: &'_ ExprTuple) {
        todo!()
    }

    fn visit_expr_unary(&mut self, i: &'_ ExprUnary) {
        todo!()
    }

    fn visit_expr_unsafe(&mut self, i: &'_ ExprUnsafe) {
        todo!()
    }

    fn visit_expr_while(&mut self, i: &'_ ExprWhile) {
        todo!()
    }

    fn visit_expr_yield(&mut self, i: &'_ ExprYield) {
        todo!()
    }

    fn visit_field(&mut self, i: &'_ Field) {
        todo!()
    }

    fn visit_field_mutability(&mut self, i: &'_ FieldMutability) {
        todo!()
    }

    fn visit_field_pat(&mut self, i: &'_ FieldPat) {
        todo!()
    }

    fn visit_field_value(&mut self, i: &'_ FieldValue) {
        todo!()
    }

    fn visit_fields(&mut self, i: &'_ Fields) {
        todo!()
    }

    fn visit_fields_named(&mut self, i: &'_ FieldsNamed) {
        todo!()
    }

    fn visit_fields_unnamed(&mut self, i: &'_ FieldsUnnamed) {
        todo!()
    }

    fn visit_file(&mut self, i: &'_ File) {
        todo!()
    }

    fn visit_fn_arg(&mut self, i: &'_ FnArg) {
        todo!()
    }

    fn visit_foreign_item(&mut self, i: &'_ ForeignItem) {
        todo!()
    }

    fn visit_foreign_item_fn(&mut self, i: &'_ ForeignItemFn) {
        todo!()
    }

    fn visit_foreign_item_macro(&mut self, i: &'_ ForeignItemMacro) {
        todo!()
    }

    fn visit_foreign_item_static(&mut self, i: &'_ ForeignItemStatic) {
        todo!()
    }

    fn visit_foreign_item_type(&mut self, i: &'_ ForeignItemType) {
        todo!()
    }

    fn visit_generic_argument(&mut self, i: &'_ GenericArgument) {
        todo!()
    }

    fn visit_generic_param(&mut self, i: &'_ GenericParam) {
        todo!()
    }

    fn visit_generics(&mut self, i: &'_ Generics) {
        todo!()
    }

    fn visit_ident(&mut self, i: &'_ Ident) {
        todo!()
    }

    fn visit_impl_item(&mut self, i: &'_ ImplItem) {
        todo!()
    }

    fn visit_impl_item_const(&mut self, i: &'_ ImplItemConst) {
        todo!()
    }

    fn visit_impl_item_fn(&mut self, i: &'_ ImplItemFn) {
        todo!()
    }

    fn visit_impl_item_macro(&mut self, i: &'_ ImplItemMacro) {
        todo!()
    }

    fn visit_impl_item_type(&mut self, i: &'_ ImplItemType) {
        todo!()
    }

    fn visit_impl_restriction(&mut self, i: &'_ ImplRestriction) {
        todo!()
    }

    fn visit_index(&mut self, i: &'_ Index) {
        todo!()
    }

    fn visit_item(&mut self, i: &'_ Item) {
        todo!()
    }

    fn visit_item_const(&mut self, i: &'_ ItemConst) {
        todo!()
    }

    fn visit_item_enum(&mut self, i: &'_ ItemEnum) {
        todo!()
    }

    fn visit_item_extern_crate(&mut self, i: &'_ ItemExternCrate) {
        todo!()
    }

    fn visit_item_fn(&mut self, i: &'_ ItemFn) {
        todo!()
    }

    fn visit_item_foreign_mod(&mut self, i: &'_ ItemForeignMod) {
        todo!()
    }

    fn visit_item_impl(&mut self, i: &'_ ItemImpl) {
        todo!()
    }

    fn visit_item_macro(&mut self, i: &'_ ItemMacro) {
        todo!()
    }

    fn visit_item_mod(&mut self, i: &'_ ItemMod) {
        todo!()
    }

    fn visit_item_static(&mut self, i: &'_ ItemStatic) {
        todo!()
    }

    fn visit_item_struct(&mut self, i: &'_ ItemStruct) {
        todo!()
    }

    fn visit_item_trait(&mut self, i: &'_ ItemTrait) {
        todo!()
    }

    fn visit_item_trait_alias(&mut self, i: &'_ ItemTraitAlias) {
        todo!()
    }

    fn visit_item_type(&mut self, i: &'_ ItemType) {
        todo!()
    }

    fn visit_item_union(&mut self, i: &'_ ItemUnion) {
        todo!()
    }

    fn visit_item_use(&mut self, i: &'_ ItemUse) {
        todo!()
    }

    fn visit_label(&mut self, i: &'_ Label) {
        todo!()
    }

    fn visit_lifetime(&mut self, i: &'_ Lifetime) {
        todo!()
    }

    fn visit_lifetime_param(&mut self, i: &'_ LifetimeParam) {
        todo!()
    }

    fn visit_lit(&mut self, i: &'_ Lit) {
        todo!()
    }

    fn visit_lit_bool(&mut self, i: &'_ LitBool) {
        todo!()
    }

    fn visit_lit_byte(&mut self, i: &'_ LitByte) {
        todo!()
    }

    fn visit_lit_byte_str(&mut self, i: &'_ LitByteStr) {
        todo!()
    }

    fn visit_lit_char(&mut self, i: &'_ LitChar) {
        todo!()
    }

    fn visit_lit_float(&mut self, i: &'_ LitFloat) {
        todo!()
    }

    fn visit_lit_int(&mut self, i: &'_ LitInt) {
        todo!()
    }

    fn visit_lit_str(&mut self, i: &'_ LitStr) {
        todo!()
    }

    fn visit_local(&mut self, i: &'_ Local) {
        todo!()
    }

    fn visit_local_init(&mut self, i: &'_ LocalInit) {
        todo!()
    }

    fn visit_macro(&mut self, i: &'_ Macro) {
        todo!()
    }

    fn visit_macro_delimiter(&mut self, i: &'_ MacroDelimiter) {
        todo!()
    }

    fn visit_member(&mut self, i: &'_ Member) {
        todo!()
    }

    fn visit_meta(&mut self, i: &'_ Meta) {
        todo!()
    }

    fn visit_meta_list(&mut self, i: &'_ MetaList) {
        todo!()
    }

    fn visit_meta_name_value(&mut self, i: &'_ MetaNameValue) {
        todo!()
    }

    fn visit_parenthesized_generic_arguments(&mut self, i: &'_ ParenthesizedGenericArguments) {
        todo!()
    }

    fn visit_pat(&mut self, i: &'_ Pat) {
        todo!()
    }

    fn visit_pat_ident(&mut self, i: &'_ PatIdent) {
        todo!()
    }

    fn visit_pat_or(&mut self, i: &'_ PatOr) {
        todo!()
    }

    fn visit_pat_paren(&mut self, i: &'_ PatParen) {
        todo!()
    }

    fn visit_pat_reference(&mut self, i: &'_ PatReference) {
        todo!()
    }

    fn visit_pat_rest(&mut self, i: &'_ PatRest) {
        todo!()
    }

    fn visit_pat_slice(&mut self, i: &'_ PatSlice) {
        todo!()
    }

    fn visit_pat_struct(&mut self, i: &'_ PatStruct) {
        todo!()
    }

    fn visit_pat_tuple(&mut self, i: &'_ PatTuple) {
        todo!()
    }

    fn visit_pat_tuple_struct(&mut self, i: &'_ PatTupleStruct) {
        todo!()
    }

    fn visit_pat_type(&mut self, i: &'_ PatType) {
        todo!()
    }

    fn visit_pat_wild(&mut self, i: &'_ PatWild) {
        todo!()
    }

    fn visit_path(&mut self, i: &'_ Path) {
        todo!()
    }

    fn visit_path_arguments(&mut self, i: &'_ PathArguments) {
        todo!()
    }

    fn visit_path_segment(&mut self, i: &'_ PathSegment) {
        todo!()
    }

    fn visit_predicate_lifetime(&mut self, i: &'_ PredicateLifetime) {
        todo!()
    }

    fn visit_predicate_type(&mut self, i: &'_ PredicateType) {
        todo!()
    }

    fn visit_qself(&mut self, i: &'_ QSelf) {
        todo!()
    }

    fn visit_range_limits(&mut self, i: &'_ RangeLimits) {
        todo!()
    }

    fn visit_receiver(&mut self, i: &'_ Receiver) {
        todo!()
    }

    fn visit_return_type(&mut self, i: &'_ ReturnType) {
        todo!()
    }

    fn visit_signature(&mut self, i: &'_ Signature) {
        todo!()
    }

    fn visit_span(&mut self, i: &Span) {
        todo!()
    }

    fn visit_static_mutability(&mut self, i: &'_ StaticMutability) {
        todo!()
    }

    fn visit_stmt(&mut self, i: &'_ Stmt) {
        todo!()
    }

    fn visit_stmt_macro(&mut self, i: &'_ StmtMacro) {
        todo!()
    }

    fn visit_trait_bound(&mut self, i: &'_ TraitBound) {
        todo!()
    }

    fn visit_trait_bound_modifier(&mut self, i: &'_ TraitBoundModifier) {
        todo!()
    }

    fn visit_trait_item(&mut self, i: &'_ TraitItem) {
        todo!()
    }

    fn visit_trait_item_const(&mut self, i: &'_ TraitItemConst) {
        todo!()
    }

    fn visit_trait_item_fn(&mut self, i: &'_ TraitItemFn) {
        todo!()
    }

    fn visit_trait_item_macro(&mut self, i: &'_ TraitItemMacro) {
        todo!()
    }

    fn visit_trait_item_type(&mut self, i: &'_ TraitItemType) {
        todo!()
    }

    fn visit_type(&mut self, i: &'_ Type) {
        todo!()
    }

    fn visit_type_array(&mut self, i: &'_ TypeArray) {
        todo!()
    }

    fn visit_type_bare_fn(&mut self, i: &'_ TypeBareFn) {
        todo!()
    }

    fn visit_type_group(&mut self, i: &'_ TypeGroup) {
        todo!()
    }

    fn visit_type_impl_trait(&mut self, i: &'_ TypeImplTrait) {
        todo!()
    }

    fn visit_type_infer(&mut self, i: &'_ TypeInfer) {
        todo!()
    }

    fn visit_type_macro(&mut self, i: &'_ TypeMacro) {
        todo!()
    }

    fn visit_type_never(&mut self, i: &'_ TypeNever) {
        todo!()
    }

    fn visit_type_param(&mut self, i: &'_ TypeParam) {
        todo!()
    }

    fn visit_type_param_bound(&mut self, i: &'_ TypeParamBound) {
        todo!()
    }

    fn visit_type_paren(&mut self, i: &'_ TypeParen) {
        todo!()
    }

    fn visit_type_path(&mut self, i: &'_ TypePath) {
        todo!()
    }

    fn visit_type_ptr(&mut self, i: &'_ TypePtr) {
        todo!()
    }

    fn visit_type_reference(&mut self, i: &'_ TypeReference) {
        todo!()
    }

    fn visit_type_slice(&mut self, i: &'_ TypeSlice) {
        todo!()
    }

    fn visit_type_trait_object(&mut self, i: &'_ TypeTraitObject) {
        todo!()
    }

    fn visit_type_tuple(&mut self, i: &'_ TypeTuple) {
        todo!()
    }

    fn visit_un_op(&mut self, i: &'_ UnOp) {
        todo!()
    }

    fn visit_use_glob(&mut self, i: &'_ UseGlob) {
        todo!()
    }

    fn visit_use_group(&mut self, i: &'_ UseGroup) {
        todo!()
    }

    fn visit_use_name(&mut self, i: &'_ UseName) {
        todo!()
    }

    fn visit_use_path(&mut self, i: &'_ UsePath) {
        todo!()
    }

    fn visit_use_rename(&mut self, i: &'_ UseRename) {
        todo!()
    }

    fn visit_use_tree(&mut self, i: &'_ UseTree) {
        todo!()
    }

    fn visit_variadic(&mut self, i: &'_ Variadic) {
        todo!()
    }

    fn visit_variant(&mut self, i: &'_ Variant) {
        todo!()
    }

    fn visit_vis_restricted(&mut self, i: &'_ VisRestricted) {
        todo!()
    }

    fn visit_visibility(&mut self, i: &'_ Visibility) {
        todo!()
    }

    fn visit_where_clause(&mut self, i: &'_ WhereClause) {
        todo!()
    }

    fn visit_where_predicate(&mut self, i: &'_ WherePredicate) {
        todo!()
    }
}
