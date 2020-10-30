use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Token, Type, braced, parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, token::{self}};

#[proc_macro_derive(ClassFromStr)]
pub fn derive_from_str(input : TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ = input.ident;
    let mut str_ident = struct_.to_string();
    if str_ident.ends_with("_") {
        let _ = str_ident.pop();
    }
    let fname = format!("CLASSNAME_{}",struct_.to_string());
    let field_name =  Ident::new(&fname,Span::call_site());

    let expanded = quote! {
        
        lazy_static! {
            static ref #field_name : String = format!("{}/{}",PACKAGE_NAME.to_string(), #str_ident);
        }

        impl AsRef<str> for #struct_ {
            fn as_ref(&self) -> &str {
                #field_name.as_str()
            }
        }
        impl Into<String> for #struct_ {
            fn into(self) -> String { 
                #field_name.to_string()
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Instance)]
pub fn derive_instance(input : TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ = input.ident;

    let expanded = quote! {
        impl<'a> Instance for #struct_ <'a> {
            fn get_obj(&self) -> Result<jni::objects::JObject> {
                Ok(self.0.l()?)
            }
        }
    };
    TokenStream::from(expanded)
}

static JEB_JAVA_PATH : &str = "./src/jeb/java";
static JEB_PATH : &str = env!("JEB_PATH", "you need to define the path to the JEB");

#[proc_macro]
pub fn define_jclass(input : TokenStream) -> TokenStream {
    let java_class = parse_macro_input!(input as JavaClass);
    std::fs::create_dir_all(JEB_JAVA_PATH).unwrap();
    let package_segments = java_class.package_name.segments;
    let package_name : String = quote!{#package_segments}.to_string().replace(" ", "");
    let mut body = "".to_string();
    body += &format!("private final long ptr; \npublic {}(long ptr) {{ this.ptr = ptr; }}\n", java_class.class_name.to_string());
    let mut native_methods = vec![];
    for method in &java_class.body {
        body += &format!("public static native {} {}Native (", method.ty, method.fname);
        let mut arguments = "".to_string();
        let mut without_types = "".to_string();
        let mut signature = "(".to_string();
        let mut args_for_native = vec![];
        for arg in &method.args {
            let typ = &arg.ty.segments;
            let typ = quote!{#typ}.to_string().replace(" ", "");
            let arg = &arg.arg;
            let arg = quote!{#arg}.to_string().replace(" ", "");
            without_types += &format!("{},", arg);
            arguments+= &format!("{} {},", typ, arg);
            args_for_native.push(format_ident!("{}", arg));
            signature += &format!("L{};", typ);
        }
        signature += "J)V";
        native_methods.push((format!("{}_rust", method.fname), format!("{}Native", method.fname), method.fname.clone(), signature, args_for_native));
        arguments.pop();
        without_types.pop();
        body += &format!("{}, long instancePtr);\n", arguments);
       
        body +=  &format!("public {} {} ({}) {{ {}Native({}, this.ptr);  }}\n\n", method.ty, method.fname, arguments, method.fname, without_types);
    }
    

    let interfaces = java_class.interface;
    let interfaces : String = quote!{#interfaces}.to_string().replace(" ", ""); 

    let body = format!("package {};\n public class {} implements {} {{\n {}\n }} ",package_name, java_class.class_name, interfaces, body);
    let output_path = format!("{}/{}.java",JEB_JAVA_PATH,java_class.class_name.to_string());
    std::fs::write(&output_path, body).unwrap();

    let output = std::process::Command::new("javac").args(&["-cp", JEB_PATH, &format!("{}/{}.java", JEB_JAVA_PATH, java_class.class_name.to_string())]).output().expect("could not execute program");
    println!("{:?}", output);
    let register_name = format_ident!("REGISTER_{}", inflector::cases::screamingsnakecase::to_screaming_snake_case(java_class.class_name.to_string().as_str()));
    let rust_class_name = format_ident!("{}", java_class.class_name);
    let rust_class_name_ = format_ident!("{}_", java_class.class_name);
    let rust_trait_name = format_ident!("I{}", java_class.class_name);
    let full_class_name = format!("{}.{}", package_name, java_class.class_name.to_string());
    let build_output = format!("java/{}.class", java_class.class_name.to_string());
   
    let package_name_for_static = package_name.replace(".", "/");

    let mut rust_functions = vec![];
    let mut jni_registers = vec![];
    let mut trait_functions = vec![];
    for ele in native_methods {
        let pkg_name = package_name.replace(".", "_");
        let rust_function_name = format_ident!("{}_{}_{}", inflector::cases::snakecase::to_snake_case(&pkg_name),inflector::cases::snakecase::to_snake_case(&rust_class_name.to_string()),inflector::cases::snakecase::to_snake_case(&ele.0));
        let trait_function = format_ident!("{}",inflector::cases::snakecase::to_snake_case(ele.2.to_string().as_str()));

        let args = ele.4;
        let rust_function = quote! {
            #[no_mangle]
            pub extern "system" fn #rust_function_name(env : jni::JNIEnv,
                                                obj :  jni::objects::JObject,
                                                #(#args : jni::objects::JObject,)*
                                                rust_instance : jni::sys::jlong) {
                let rust_instance = unsafe {&mut *(rust_instance as *mut #rust_class_name) };
                rust_instance.#trait_function(#(#args,)*);
            }
        };
        rust_functions.push(rust_function);

        let trait_func = quote!{
            fn #trait_function(&self, 
                                #(#args : jni::objects::JObject),*);
        };
        trait_functions.push(trait_func);

        let native_name = ele.1;
        let ctor = ele.3;
        let jni_register = quote!{

            let native_method = jni::NativeMethod {
                name : #native_name.into(),
                sig : normalize!(#ctor).into(),
                fn_ptr : #rust_function_name as *mut _
                
            };
            native_methods.push(native_method);
            
        };
        jni_registers.push(jni_register);
    }
    let mut additional_args = vec![];
    let mut additional_ty = vec![];
    let mut assignments = vec![];
    
    for arg in java_class.additional_args {
        let name = arg.name;
        let ty = arg.ty;
        let head = quote!{ #name : #ty};
        additional_args.push(head);
        let ty = quote! {#ty};
        additional_ty.push(ty);

        let assignment = quote!{#name};
        assignments.push(assignment);
    }
    let result = quote! {
        static #register_name: std::sync::Once = std::sync::Once::new();
        static PACKAGE_NAME : &str = #package_name_for_static;
       
        #[derive(Instance)]
        pub struct #rust_class_name<'a>(pub jni::objects::JValue<'a>, #(#additional_ty,)*);
        #[derive(ClassFromStr)]
        struct #rust_class_name_;

        pub trait #rust_trait_name : Instance {
            #(#trait_functions)*
        }
        
        impl <'a> #rust_class_name<'a> {
            pub fn new(#(#additional_args,)*) -> Result<'a, Box<dyn #rust_trait_name + 'a>> {
               unsafe { #register_name.call_once(|| {
                        let bytes = include_bytes!(#build_output);
                        let env = get_vm_unwrap!();
                        let class = env.find_class(normalize!("com.pnfsoftware.jeb.core.dao.IUserDatabase")).expect("string not found");
                        let class_loader = env.call_method(class, "getClassLoader", "()Ljava/lang/ClassLoader;", &[]).expect("could not get classloader");
                        let class = env.define_class(&format!("{}", normalize!(#full_class_name)), class_loader.l().expect("class_loader is not an object"), bytes).expect("failed to register class");
                        let mut native_methods = vec![];
                        
                        #(#jni_registers)*

                        env.register_native_methods(class,&native_methods).expect("could not register native method");
                    });
                };
                let env = get_vm!();
                let mut obj = Box::new(
                    #rust_class_name(jni::objects::JObject::null().into(),#(#assignments,)*)
                );
                let raw_ptr = obj.as_mut() as *mut _ as  jni::sys::jlong;
                
                let res = env.new_object(normalize!(#full_class_name), "(J)V", &[raw_ptr.into()])?;
                obj.0 = res.into();
                Ok(obj)
            }
        }

        #(#rust_functions)*
    };
    TokenStream::from(result)
}

struct JavaClass {
    additional_args : Vec<RustArg>,
    class_name : Ident,
    package_name : Fqdn,
    interface : Punctuated<Fqdn, Token![,]>,
    body : Punctuated<JavaMethod,Token![;]>
}
struct RustArg {
    name : Ident,
    ty : Type
}

struct JavaMethod {
    _public_token : kw::public,
    _native_token :kw::native,
    ty : Ident,
    fname : Ident,
    _paren_token : token::Paren,
    args : Punctuated<TypeDeclaration, Token![,]>
}
struct TypeDeclaration {
    ty : Fqdn,
    arg : Ident
}
struct FunctionArguments {
    _args : Punctuated<TypeDeclaration, Token![,]>
}

impl Parse for JavaMethod {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let body;
        Ok(JavaMethod {
            _public_token : input.parse()?,
            _native_token : input.parse()?,
            ty : input.parse()?,
            fname : input.parse()?,
            _paren_token : parenthesized!(body in input),
            args : body.parse_terminated::<TypeDeclaration, Token![,]>(TypeDeclaration::parse)?

        })
    }
}



impl Parse for JavaClass {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = vec![];
        let par_args;
        parenthesized!(par_args in input);
        while !par_args.is_empty() {
            let name : Ident = par_args.parse()?;
            let _ : Token![:] = par_args.parse()?;
            let ty : Type = par_args.parse()?;
            args.push(RustArg{name, ty});
            if par_args.peek(Token![,]) && par_args.peek2(syn::Ident) {
                par_args.parse::<Token![,]>()?;
            } else if par_args.peek(Token![,]) {
                break;
            }
        }
        input.parse::<kw::package>()?;
        let package_name : Fqdn = input.parse()?;
        input.parse::<Token![;]>()?;

        input.parse::<kw::public>()?;
        input.parse::<kw::class>()?;
        let class_name : Ident = input.parse()?;
       
        input.parse::<kw::implements>()?;
        let mut interface = Punctuated::new();
        let first:  Fqdn = input.parse()?;
        interface.push(first);
        while input.peek(Token![,]) {
            let punct = input.parse::<Token![,]>()?;
            interface.push_punct(punct);
            let next : Fqdn = input.parse()?;
            interface.push(next);
        }
      
        let body;
        braced!(body in input);
        let body = body.parse_terminated(JavaMethod::parse)?;

        Ok(
            JavaClass{
                additional_args : args,
                package_name,
                class_name ,
                interface,
                body 
            }
        )
    }
}

struct Fqdn {
    segments : Punctuated<Ident, punc::PackageSeparator>
}

impl ToTokens for Fqdn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.segments.to_tokens(tokens);
    }
}

impl Parse for Fqdn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut segments = Punctuated::new();
        let first : Ident = input.parse()?;
        segments.push(first);
        while input.peek(punc::PackageSeparator) {
            segments.push_punct(input.parse()?);
            let next : Ident = input.parse()?;
            segments.push_value(next);
        }
       
        Ok(Fqdn{segments})
    }
}
impl Parse for FunctionArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Punctuated::new();
        let first : TypeDeclaration = input.parse()?;
        args.push(first);
        while input.peek(Token![,]) {
            args.push_punct(input.parse()?);
            let next : TypeDeclaration = input.parse()?;
            args.push_value(next);
        }
       
        Ok(FunctionArguments{_args : args})
    }
}
impl Parse for TypeDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty : Fqdn = input.parse()?;
        let arg : Ident = input.parse()?;
        Ok(
            TypeDeclaration{
                ty,
                arg
            }
        )
    }
}


#[allow(dead_code)]
mod kw {
    use syn;

    syn::custom_keyword!(package);
    syn::custom_keyword!(implements);
    syn::custom_keyword!(native);
    syn::custom_keyword!(class);
    syn::custom_keyword!(void);
    syn::custom_keyword!(int);
    syn::custom_keyword!(long);
    syn::custom_keyword!(public);
}
mod punc {
    syn::custom_punctuation!(PackageSeparator,.);
    
}